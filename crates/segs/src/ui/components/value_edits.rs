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
        .with_width(50.) // Fine-tuned to fit 65535
        .char_limit(5)
}
