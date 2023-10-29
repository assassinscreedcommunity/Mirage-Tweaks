pub fn show(error: String) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Mirage Tweaks")
        .set_description(error)
        .show();
}
