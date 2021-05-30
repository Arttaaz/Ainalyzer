use druid::Widget;
use druid::widget::{Button, Flex};

pub fn create_ask_save_changes_dialog() -> impl Widget<crate::RootState> {
    let b_save: Button<crate::RootState> = Button::new("Save");
    let b_save = b_save.on_click(|c, data, _| {
        if data.is_file_updated() {
            c.submit_command(druid::Command::new(druid::commands::SAVE_FILE, (), druid::Target::Auto));
        } else {
            let open_options = druid::FileDialogOptions::new()
                .allowed_types(vec![druid::FileSpec::new("sgf", &["sgf"])]);
            c.submit_command(druid::Command::new(druid::commands::SHOW_SAVE_PANEL, open_options, druid::Target::Auto));
        }
        c.submit_command(druid::Command::new(druid::commands::CLOSE_WINDOW, (), druid::Target::Auto));
    });

    let b_discard: Button<crate::RootState> = Button::new("Discard");
    let b_discard = b_discard.on_click(|c, _, _| {
        c.submit_command(druid::Command::new(druid::Selector::new("ainalyzer.new_file"), (), druid::Target::Auto));
        c.submit_command(druid::Command::new(druid::commands::CLOSE_WINDOW, (), druid::Target::Auto));
    });


    let b_cancel: Button<crate::RootState> = Button::new("Cancel");
    let b_cancel = b_cancel.on_click(|c, _, _| {
        c.submit_command(druid::Command::new(druid::commands::CLOSE_WINDOW, (), druid::Target::Auto));
    });

    let layout = Flex::row()
        .with_flex_spacer(crate::HORIZONTAL_WIDGET_SPACING + 0.9)
        .with_flex_child(b_save, 1.0)
        .with_flex_child(b_discard, 1.0)
        .with_flex_child(b_cancel, 1.0);
//        .with_flex_spacer(crate::HORIZONTAL_WIDGET_SPACING);

    let layout2 = Flex::column()
        .with_flex_child(druid::widget::Label::new("Would you like to save your changes?"), 0.5)
        .with_spacer(crate::VERTICAL_WIDGET_SPACING)
        .with_child(layout);

    Flex::row()
        .with_flex_spacer(crate::HORIZONTAL_WIDGET_SPACING)
        .with_flex_child(layout2, 1.0)
        .with_flex_spacer(crate::HORIZONTAL_WIDGET_SPACING)
}
