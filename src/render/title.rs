use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::memory::{
    MemoryRenderBuffer, MemoryRenderBufferRenderElement,
};
use smithay::backend::renderer::element::texture::TextureBuffer;
use smithay::backend::renderer::element::{Element, Kind, RenderElement};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture, ffi};
use smithay::backend::renderer::{Bind, Color32F, Frame, Offscreen, Renderer, Texture};
use smithay::utils::{Physical, Point, Rectangle, Size, Transform};

use super::{
    OutputRenderElements, PixelSnapRescaleElement, TrimmedElement, WindowRenderAnimation,
    WindowTransformElement,
};

#[derive(Clone)]
pub(crate) enum TitlePixels {
    Memory(MemoryRenderBuffer),
    Texture(TextureBuffer<GlesTexture>, Size<i32, Physical>),
}

impl TitlePixels {
    pub(crate) fn element(
        &self,
        renderer: &mut GlesRenderer,
        rect: Rectangle<i32, Physical>,
        scale: f64,
        zoom: f64,
        opacity: f64,
        animation: Option<WindowRenderAnimation>,
    ) -> Option<OutputRenderElements> {
        match self {
            Self::Memory(buffer) => {
                let element = MemoryRenderBufferRenderElement::from_buffer(
                    renderer,
                    rect.loc.to_f64(),
                    buffer,
                    Some(opacity as f32),
                    None,
                    None,
                    Kind::Unspecified,
                )
                .ok()?;
                let element = PixelSnapRescaleElement::from_element(
                    TrimmedElement::from_element(element, rect),
                    Point::default(),
                    zoom,
                );
                Some(if let Some(a) = animation {
                    OutputRenderElements::AnimatedTrimmedDecoration(WindowTransformElement::new(
                        element, a.origin, a.offset, a.scale,
                    ))
                } else {
                    OutputRenderElements::TrimmedDecoration(element)
                })
            }
            Self::Texture(buffer, texels) => {
                let rect = Rectangle::new(
                    rect.loc.to_f64().upscale(zoom),
                    rect.size.to_f64().upscale(zoom),
                )
                .to_i32_round::<i32>();
                let rect = animation.map_or(rect, |a| a.transform_phys_rect(rect));
                Some(super::ResizeCrossfade::render_retained_rect(
                    buffer,
                    *texels,
                    rect.to_f64(),
                    scale,
                    opacity,
                ))
            }
        }
    }
}

pub(crate) struct TitleCrossfade {
    source: TitlePixels,
    mixed: Option<GlesTexture>,
}

impl TitleCrossfade {
    pub(crate) fn new(source: TitlePixels) -> Self {
        Self {
            source,
            mixed: None,
        }
    }

    pub(crate) fn retained(&self, renderer: &GlesRenderer) -> TitlePixels {
        self.mixed.as_ref().map_or_else(
            || self.source.clone(),
            |texture| {
                TitlePixels::Texture(
                    TextureBuffer::from_texture(
                        renderer,
                        texture.clone(),
                        1,
                        Transform::Normal,
                        None,
                    ),
                    Size::from((texture.size().w, texture.size().h)),
                )
            },
        )
    }

    pub(crate) fn blend(
        &mut self,
        renderer: &mut GlesRenderer,
        current: &MemoryRenderBuffer,
        size: Size<i32, Physical>,
        scale: f64,
        weight: f32,
    ) -> Option<TitlePixels> {
        if weight == 1.0 {
            return Some(self.source.clone());
        }
        let rect = Rectangle::from_size(size);
        let old = self
            .source
            .element(renderer, rect, scale, 1.0, f64::from(weight), None)?;
        let new = TitlePixels::Memory(current.clone()).element(
            renderer,
            rect,
            scale,
            1.0,
            f64::from(1.0 - weight),
            None,
        )?;
        if self
            .mixed
            .as_ref()
            .is_none_or(|t| t.size().w != size.w || t.size().h != size.h)
        {
            self.mixed = Some(
                Offscreen::<GlesTexture>::create_buffer(
                    renderer,
                    Fourcc::Abgr8888,
                    size.to_logical(1).to_buffer(1, Transform::Normal),
                )
                .ok()?,
            );
        }
        let texture = self.mixed.as_mut()?;
        let mut target = renderer.bind(texture).ok()?;
        let mut frame = renderer.render(&mut target, size, Transform::Normal).ok()?;
        frame.clear(Color32F::TRANSPARENT, &[rect]).ok()?;
        frame
            .with_context(|gl| unsafe {
                gl.BlendFunc(ffi::ONE, ffi::ONE);
            })
            .ok()?;
        let result = (|| {
            for element in [&old, &new] {
                element.draw(&mut frame, element.src(), rect, &[rect], &[], None)?;
            }
            Ok::<_, smithay::backend::renderer::gles::GlesError>(())
        })();
        frame
            .with_context(|gl| unsafe {
                gl.BlendFunc(ffi::ONE, ffi::ONE_MINUS_SRC_ALPHA);
            })
            .ok()?;
        result.ok()?;
        let _ = frame.finish().ok()?;
        drop(target);
        Some(TitlePixels::Texture(
            TextureBuffer::from_texture(renderer, texture.clone(), 1, Transform::Normal, None),
            size,
        ))
    }
}
