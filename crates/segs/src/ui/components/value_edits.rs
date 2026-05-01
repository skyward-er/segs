use std::net::Ipv4Addr;

use segs_ui::widgets::text::ValueEdit;

pub fn ip_value_edit<'a>(addr: &'a mut Ipv4Addr) -> ValueEdit<'a, Ipv4Addr> {
    ValueEdit::new(addr)
        .hint_text("IP Address...")
        .with_width(107.) // Fine-tuned to fit 255.255.255.255.255
        .char_limit(15)
}

pub fn port_value_edit<'a>(port: &'a mut u16) -> ValueEdit<'a, u16> {
    ValueEdit::new(port)
        .hint_text("Port...")
        .horizontal_align(egui::Align::Center)
        .with_width(54.) // Fine-tuned to fit 65535
        .char_limit(5)
}

pub fn tty_value_edit<'a>(tty: &'a mut String) -> ValueEdit<'a, String> {
    ValueEdit::new(tty)
        .hint_text("TTY Device...")
        .with_width(107.) // Fine-tuned to fit typical TTY device paths
        .char_limit(255)
}

pub fn baudrate_value_edit<'a>(baudrate: &'a mut u32) -> ValueEdit<'a, u32> {
    ValueEdit::new(baudrate)
        .hint_text("Baud Rate...")
        .horizontal_align(egui::Align::Center)
        .with_width(64.) // Fine-tuned to fit common baud rates like 115200
        .char_limit(7)
}
