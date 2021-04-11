// How to create a toast without using this library

extern crate xml;
use std::path::Path;
use xml::escape::escape_str_attribute;

#[allow(dead_code)]
mod bindings {
    ::windows::include_bindings!();
}

// You need to have the windows crate in your Cargo.toml
//
// and call windows::build! in a build.rs file
// or have pregenerated code that does the same thing
use bindings::{
    Windows::Data::Xml::Dom::XmlDocument,
    Windows::Foundation::TypedEventHandler,
    Windows::UI::Notifications::ToastActivatedEventArgs,
    Windows::UI::Notifications::ToastNotification,
    Windows::UI::Notifications::ToastNotificationManager,
    Windows::UI::Notifications::{ToastDismissalReason, ToastDismissedEventArgs, ToastFailedEventArgs},
};

//https://social.msdn.microsoft.com/Forums/Windows/en-US/99e0d4bd-07cb-4ebd-8c92-c44ac6e7e5de/toast-notification-dismissed-event-handler-not-called-every-time?forum=windowsgeneraldevelopmentissues
pub use windows::{Error, HString, Interface, Object};

fn main() {
    do_toast().expect("not sure if this is actually failable");
    // this is a hack to workaround toasts not showing up if the application closes too quickly
    // you can put this in do_toast if you want.
    std::thread::sleep(std::time::Duration::from_millis(10000));
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn do_toast() -> windows::Result<()> {
    let toast_xml = XmlDocument::new()?;

    toast_xml.LoadXml(HString::from(
        format!(r#"<toast duration="short">
                <visual>
                    <binding template="ToastGeneric">
                        <text id="1">title</text>
                        <text id="2">first line</text>
                        <text id="3">third line</text>
                        <image placement="appLogoOverride" hint-crop="circle" src="file:///c:/path_to_image_above_toast.jpg" alt="alt text" />
                        <image placement="Hero" src="file:///C:/path_to_image_in_toast.jpg" alt="alt text2" />
                        <image id="1" src="file:///{}" alt="another_image" />
                    </binding>
                </visual>
                <audio src="ms-winsoundevent:Notification.SMS" />
                <!-- <audio silent="true" /> -->
                <!-- See https://docs.microsoft.com/en-us/windows/uwp/design/shell/tiles-and-notifications/toast-pending-update?tabs=xml for possible actions -->
                <actions>
                    <action content="left" arguments="first" />
                    <action content="right" arguments="second" />
                </actions>
            </toast>"#,
        escape_str_attribute(&Path::new("C:\\path_to_image_in_toast.jpg").display().to_string()),
    ))).expect("the xml is malformed");

    // Create the toast and attach event listeners
    let toast_notification = ToastNotification::CreateToastNotification(toast_xml)?;

    // happens if any of the toasts actions are interacted with (as a popup or in the action center)
    toast_notification.Activated(TypedEventHandler::<ToastNotification, Object>::new(|sender, result| {
        // Activated has the wrong type signature so you have to cast the object
        // Dismissed and Failed have the correct signature so they work without doing this
        if let Some(obj) = &*result {
            let args = obj.cast::<ToastActivatedEventArgs>()?;
            println!("{}", args.Arguments()?);
        }

        Ok(())
    }))?;

    // happens if the toast is moved to the action center or dismissed in the action center
    // or if it ends without the user clicking on anything.
    // Note that Dismissed and then Activated can be triggered from the same toast.
    toast_notification.Dismissed(TypedEventHandler::<ToastNotification, ToastDismissedEventArgs>::new(
        |_sender, result| {

            if let Some(args) = &*result {
                let reason = match args.Reason() {
                    Ok(ToastDismissalReason::UserCanceled) => "User Canceled",
                    Ok(ToastDismissalReason::ApplicationHidden) => "Application Hidden",
                    Ok(ToastDismissalReason::TimedOut) => "Timed out",
                    Ok(_) => "Unkown reason",
                    Err(_) => "Error",
                };
                println!("{}", reason);
            };
            Ok(())
        },
    ))?;

    // happens if toasts are disabled
    toast_notification.Failed(TypedEventHandler::<ToastNotification, ToastFailedEventArgs>::new(
        |_sender, result| {
            println!("failed");
            if let Some(args) = &*result {
                println!("{}", args.ErrorCode()?.message())
            }
            Ok(())
        },
    ))?;

    // If you have a valid app id, (ie installed using wix) then use it here.
    let toast_notifier = ToastNotificationManager::CreateToastNotifierWithId(HString::from(
        "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe",
    ))?;

    // Show the toast.
    // Note this returns success in every case, including when the toast isn't shown.
    toast_notifier.Show(&toast_notification)
}
