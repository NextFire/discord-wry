use notify_rust::Notification;
use wry::{
    application::{
        accelerator::{Accelerator, RawMods},
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        keyboard::KeyCode,
        menu::{AboutMetadata, MenuBar, MenuId, MenuItem, MenuItemAttributes},
        window::WindowBuilder,
    },
    webview::WebViewBuilder,
};

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

const APP_NAME: &str = "Discord";
const APP_URL: &str = "https://discord.com/app";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36";

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();

    let (root_menu, close_ids) = make_root_menu(APP_NAME);

    let window = WindowBuilder::new()
        .with_title(APP_NAME)
        .with_menu(root_menu)
        .with_closable(false)
        .build(&event_loop)?;

    #[allow(unused_variables)]
    let webview = WebViewBuilder::new(window)?
        .with_url(APP_URL)?
        .with_user_agent(USER_AGENT)
        .with_initialization_script(NOTIFICATIONS_IPC)
        .with_ipc_handler(|_, notif| {
            Notification::new()
                .summary(APP_NAME)
                .body(notif.as_str())
                .appname(APP_NAME)
                .icon(APP_NAME)
                .show()
                .unwrap();
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::MenuEvent { menu_id, .. } => {
                if close_ids.contains(&menu_id) {
                    #[cfg(not(target_os = "macos"))]
                    webview.window().set_minimized(true);

                    #[cfg(target_os = "macos")]
                    // https://github.com/rust-windowing/winit/blob/65ac35e3a4b5517fe042b57823a9ed36d2f1de4e/src/platform/macos.rs#L221-L225
                    {
                        let cls = objc::runtime::Class::get("NSApplication").unwrap();
                        let app: cocoa::base::id = unsafe { msg_send![cls, sharedApplication] };
                        unsafe { msg_send![app, hide: 0] }
                    }
                }
            }

            _ => (),
        }
    });
}

/// https://github.com/tauri-apps/tauri/blob/6ff801e27d972a221325c8e86cbdfddb6bb9c099/core/tauri-runtime/src/menu.rs#L245
fn make_root_menu(app_name: &str) -> (MenuBar, Vec<MenuId>) {
    let mut root_menu = MenuBar::new();

    #[cfg(target_os = "macos")]
    {
        let mut app_submenu = MenuBar::new();
        app_submenu.add_native_item(MenuItem::About(
            app_name.to_string(),
            AboutMetadata::default(),
        ));
        app_submenu.add_native_item(MenuItem::Separator);
        app_submenu.add_native_item(MenuItem::Services);
        app_submenu.add_native_item(MenuItem::Separator);
        app_submenu.add_native_item(MenuItem::Hide);
        app_submenu.add_native_item(MenuItem::HideOthers);
        app_submenu.add_native_item(MenuItem::ShowAll);
        app_submenu.add_native_item(MenuItem::Separator);
        app_submenu.add_native_item(MenuItem::Quit);
        root_menu.add_submenu(app_name, true, app_submenu);
    }

    let mut file_menu = MenuBar::new();
    // file_menu.add_native_item(MenuItem::CloseWindow);
    let close1 = file_menu.add_item(
        MenuItemAttributes::new("Close Window")
            .with_accelerators(&Accelerator::new(RawMods::Meta, KeyCode::KeyW)),
    );
    #[cfg(not(target_os = "macos"))]
    {
        file_menu.add_native_item(MenuItem::Quit);
    }
    root_menu.add_submenu("File", true, file_menu);

    #[cfg(not(target_os = "linux"))]
    let mut edit_menu = MenuBar::new();
    #[cfg(target_os = "macos")]
    {
        edit_menu.add_native_item(MenuItem::Undo);
        edit_menu.add_native_item(MenuItem::Redo);
        edit_menu.add_native_item(MenuItem::Separator);
    }
    #[cfg(not(target_os = "linux"))]
    {
        edit_menu.add_native_item(MenuItem::Cut);
        edit_menu.add_native_item(MenuItem::Copy);
        edit_menu.add_native_item(MenuItem::Paste);
    }
    #[cfg(target_os = "macos")]
    {
        edit_menu.add_native_item(MenuItem::SelectAll);
    }
    #[cfg(not(target_os = "linux"))]
    {
        root_menu.add_submenu("Edit", true, edit_menu);
    }
    #[cfg(target_os = "macos")]
    {
        let mut view_menu = MenuBar::new();
        view_menu.add_native_item(MenuItem::EnterFullScreen);
        root_menu.add_submenu("View", true, view_menu);
    }

    let mut window_menu = MenuBar::new();
    window_menu.add_native_item(MenuItem::Minimize);
    #[cfg(target_os = "macos")]
    {
        window_menu.add_native_item(MenuItem::Zoom);
        window_menu.add_native_item(MenuItem::Separator);
    }
    // window_menu.add_native_item(MenuItem::CloseWindow);
    let close2 = window_menu.add_item(
        MenuItemAttributes::new("Close Window")
            .with_accelerators(&Accelerator::new(RawMods::Meta, KeyCode::KeyW)),
    );
    root_menu.add_submenu("Window", true, window_menu);

    (root_menu, vec![close1.id(), close2.id()])
}

/// Adapted from https://github.com/peterthomashorn/wkwebviewnotificationexample/blob/efba5d9bd222b72498c451e91d7371b20eb47551/WKWebViewNotificationExample/UserScript.js
const NOTIFICATIONS_IPC: &str = r#"
/**
 * Incomplete Notification API override to enable native notifications.
 */
class NotificationOverride {
  // Grant permission by default to keep this example simple.
  // Safari 13 does not support class fields yet, so a static getter must be used.
  static get permission() {
      return "granted";
  }

  // Safari 13 still uses callbacks instead of promises.
  static requestPermission (callback) {
      callback("granted");
  }

  // Forward the notification text to the native app through the script message handler.
  constructor (messageText) {
    window.ipc.postMessage(messageText);
  }
}

// Override the global browser notification object.
window.Notification = NotificationOverride;
"#;
