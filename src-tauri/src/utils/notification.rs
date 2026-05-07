use std::borrow::Cow;
use crate::core::handle;
use my_app_i18n;
use tauri_plugin_notification::NotificationExt as _;
pub enum NotificationEvent<'a> {
    DashboardToggled,
    ClashModeChanged {
        mode: &'a str,
    },
    SystemProxyModeToggled,
    TunModeToggled,
    LightweightModeEntered,
    ProfilesReactivated,
    AppQuit,
    #[cfg(target_os = "macos")]
    AppHidden,
}


/**
 * title 和 body 方法可能既接受 &str 也接受 String。用 Cow<'_, str> 就可以：
调用方可以传 "标题"（&str，零开销）
也可以传 format!("{}", something)（String，自动接管所有权）
函数内部不需要关心是哪种，直接使用即可
相当于一个灵活的参数容器，在不需要所有权时避免不必要的克隆。
 */
fn notify(title: Cow<'_, str>, body: Cow<'_, str>) {
    let app_handle = handle::Handle::app_handle();
    app_handle.notification().builder().title(title).body(body).show().ok();
}

pub async fn notify_event<'a>(event: NotificationEvent<'a>) {
    match event {
        NotificationEvent::DashboardToggled => {
            let title = my_app_i18n::t!("notifications.dashboardToggled.title");
            let body = my_app_i18n::t!("notifications.dashboardToggled.body");
            notify(title, body);
        }
        NotificationEvent::ClashModeChanged { mode } => {
            let title = my_app_i18n::t!("notifications.clashModeChanged.title");
            let body = my_app_i18n::t!("notifications.clashModeChanged.body")
                .replace("{mode}", mode)
                .into();
            notify(title, body);
        }
        NotificationEvent::SystemProxyModeToggled => {
            let title = my_app_i18n::t!("notifications.systemProxyToggled.title");
            let body = my_app_i18n::t!("notifications.systemProxyToggled.body");
            notify(title, body);
        }
        NotificationEvent::TunModeToggled => {
            let title = my_app_i18n::t!("notifications.tunModeToggled.title");
            let body = my_app_i18n::t!("notifications.tunModeToggled.body");
            notify(title, body);
        }
        NotificationEvent::LightweightModeEntered => {
            let title = my_app_i18n::t!("notifications.lightweightModeEntered.title");
            let body = my_app_i18n::t!("notifications.lightweightModeEntered.body");
            notify(title, body);
        }
        NotificationEvent::ProfilesReactivated => {
            let title = my_app_i18n::t!("notifications.profilesReactivated.title");
            let body = my_app_i18n::t!("notifications.profilesReactivated.body");
            notify(title, body);
        }
        NotificationEvent::AppQuit => {
            let title = my_app_i18n::t!("notifications.appQuit.title");
            let body = my_app_i18n::t!("notifications.appQuit.body");
            notify(title, body);
        }
        #[cfg(target_os = "macos")]
        NotificationEvent::AppHidden => {
            let title = my_app_i18n::t!("notifications.appHidden.title");
            let body = my_app_i18n::t!("notifications.appHidden.body");
            notify(title, body);
        }
    }
}
