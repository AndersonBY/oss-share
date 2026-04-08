use tauri::{
    AppHandle, Manager,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open_files = MenuItemBuilder::with_id("open_files", "打开文件管理").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "设置").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&open_files)
        .item(&settings)
        .item(&separator)
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("OSS Share")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "open_files" => {
                show_window(app, "files");
            }
            "settings" => {
                show_window(app, "settings");
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

pub fn show_window(app: &AppHandle, tab: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.eval(&format!(
            "window.__navigateTo && window.__navigateTo('{tab}')"
        ));
    }
}
