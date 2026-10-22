use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use niri_config::{Action, Config, MruDirection};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

struct Files(PathBuf);

impl Files {
    fn new() -> Self {
        let root = PathBuf::from(std::env::var_os("DRIFTWM_REFERENCE_SCRATCH").unwrap());
        let path = root.join(NEXT_DIR.fetch_add(1, Ordering::Relaxed).to_string());
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, contents: &str) {
        std::fs::write(self.0.join(name), contents).unwrap();
    }

    fn parse(
        &self,
        text: &str,
    ) -> niri_config::ConfigParseResult<Config, niri_config::ConfigIncludeError> {
        Config::parse(&self.0.join("main.kdl"), text)
    }
}

#[test]
fn includes_apply_at_their_source_position() {
    let files = Files::new();
    files.write("child.kdl", "layout { gaps 24; }\n");
    let before = files
        .parse("layout { gaps 8; }\ninclude \"child.kdl\"\n")
        .config
        .unwrap();
    let after = files
        .parse("include \"child.kdl\"\nlayout { gaps 8; }\n")
        .config
        .unwrap();
    assert_eq!(before.layout.gaps, 24.0);
    assert_eq!(after.layout.gaps, 8.0);
}

#[test]
fn duplicate_single_sections_in_one_file_are_rejected() {
    assert!(Config::parse_mem("layout { gaps 8; }\nlayout { gaps 24; }\n").is_err());
}

#[test]
fn optional_missing_include_is_accepted_and_watched() {
    let files = Files::new();
    let parsed = files.parse("include \"missing.kdl\" optional=true\n");
    assert!(parsed.config.is_ok());
    assert_eq!(parsed.includes, vec![files.0.join("missing.kdl")]);
}

#[test]
fn required_missing_include_is_rejected_but_watched() {
    let files = Files::new();
    let parsed = files.parse("include \"missing.kdl\"\n");
    assert!(parsed.config.is_err());
    assert_eq!(parsed.includes, vec![files.0.join("missing.kdl")]);
}

#[test]
fn optional_malformed_include_rejects_the_entire_config() {
    let files = Files::new();
    files.write("bad.kdl", "layout { gaps \"not-a-number\"; }\n");
    let parsed = files.parse("layout { gaps 8; }\ninclude \"bad.kdl\" optional=true\n");
    assert!(parsed.config.is_err());
    assert_eq!(parsed.includes, vec![files.0.join("bad.kdl")]);
}

#[test]
fn optional_include_does_not_hide_other_io_errors() {
    let files = Files::new();
    std::fs::create_dir(files.0.join("directory")).unwrap();
    assert!(files
        .parse("include \"directory\" optional=true\n")
        .config
        .is_err());
}

#[test]
fn nested_includes_resolve_relative_to_their_own_file() {
    let files = Files::new();
    std::fs::create_dir(files.0.join("nested")).unwrap();
    files.write("nested/child.kdl", "include \"settings.kdl\"\n");
    files.write("nested/settings.kdl", "layout { gaps 19; }\n");
    assert_eq!(
        files
            .parse("include \"nested/child.kdl\"\n")
            .config
            .unwrap()
            .layout
            .gaps,
        19.0
    );
}

#[test]
fn recursive_include_is_rejected() {
    let files = Files::new();
    files.write("main.kdl", "include \"main.kdl\"\n");
    assert!(files.parse("include \"main.kdl\"\n").config.is_err());
}

#[test]
fn nine_include_levels_work_but_ten_are_rejected() {
    for depth in [9, 10] {
        let files = Files::new();
        for level in 1..=depth {
            let text = if level == depth {
                "layout { gaps 17; }\n".to_string()
            } else {
                format!("include \"level{}.kdl\"\n", level + 1)
            };
            files.write(&format!("level{level}.kdl"), &text);
        }
        let parsed = files.parse("include \"level1.kdl\"\n");
        assert_eq!(parsed.config.is_ok(), depth == 9);
    }
}

#[test]
fn first_empty_mru_bind_block_removes_built_in_bindings() {
    assert_eq!(Config::parse_mem("").unwrap().recent_windows.binds.len(), 8);
    assert!(Config::parse_mem("recent-windows { binds {}; }\n")
        .unwrap()
        .recent_windows
        .binds
        .is_empty());
}

#[test]
fn later_mru_bind_blocks_replace_matching_keys_and_preserve_other_keys() {
    let files = Files::new();
    files.write(
        "first.kdl",
        "recent-windows { binds { Alt+Tab { next-window; }; Mod+Tab { next-window; }; }; }\n",
    );
    files.write(
        "second.kdl",
        "recent-windows { binds { Alt+Tab { previous-window; }; }; }\n",
    );
    let config = files
        .parse("include \"first.kdl\"\ninclude \"second.kdl\"\n")
        .config
        .unwrap();
    assert_eq!(config.recent_windows.binds.len(), 2);
    let backwards = config
        .recent_windows
        .binds
        .iter()
        .filter(|b| {
            matches!(
                b.action,
                Action::MruAdvance {
                    direction: MruDirection::Backward,
                    ..
                }
            )
        })
        .count();
    assert_eq!(backwards, 1);
    files.write("second.kdl", "recent-windows { binds {}; }\n");
    assert_eq!(
        files
            .parse("include \"first.kdl\"\ninclude \"second.kdl\"\n")
            .config
            .unwrap()
            .recent_windows
            .binds
            .len(),
        2
    );
}

#[test]
fn off_wins_for_mru_but_on_wins_for_animation_and_satellite() {
    let config = Config::parse_mem(
        "recent-windows { on; off; }\nanimations { off; on; }\nxwayland-satellite { off; on; }\n",
    )
    .unwrap();
    assert!(!config.recent_windows.on);
    assert!(!config.animations.off);
    assert!(!config.xwayland_satellite.off);
}

#[test]
fn later_include_can_enable_mru_without_resetting_omitted_values() {
    let files = Files::new();
    files.write("child.kdl", "recent-windows { on; }\n");
    let config = files
        .parse("recent-windows { off; debounce-ms 123; }\ninclude \"child.kdl\"\n")
        .config
        .unwrap();
    assert!(config.recent_windows.on);
    assert_eq!(config.recent_windows.debounce_ms, 123);
}

#[test]
fn all_filter_is_accepted_explicitly_and_as_the_omitted_default() {
    for action in ["next-window;", "next-window filter=\"all\";"] {
        let config = Config::parse_mem(&format!(
            "recent-windows {{ binds {{ Alt+Tab {{ {action} }}; }}; }}\n"
        ))
        .unwrap();
        assert!(matches!(
            config.recent_windows.binds[0].action,
            Action::MruAdvance {
                filter: Some(niri_config::MruFilter::All),
                ..
            }
        ));
    }
    assert!(Config::parse_mem(
        "recent-windows { binds { Alt+Tab { next-window filter=\"unknown-filter\"; }; }; }\n"
    )
    .is_err());
}

#[test]
fn mru_bindings_require_keyboard_trigger_and_modifier() {
    for key in ["Tab", "Alt+MouseLeft"] {
        assert!(Config::parse_mem(&format!(
            "recent-windows {{ binds {{ {key} {{ next-window; }}; }}; }}\n"
        ))
        .is_err());
    }
}

#[test]
fn fresh_parse_does_not_keep_removed_cursor_settings() {
    let custom =
        Config::parse_mem("cursor { hide-when-typing; hide-after-inactive-ms 0; }\n").unwrap();
    assert!(custom.cursor.hide_when_typing);
    assert_eq!(custom.cursor.hide_after_inactive_ms, Some(0));
    let fresh = Config::parse_mem("").unwrap();
    assert!(!fresh.cursor.hide_when_typing);
    assert_eq!(fresh.cursor.hide_after_inactive_ms, None);
}
