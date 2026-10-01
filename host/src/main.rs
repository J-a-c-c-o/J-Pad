mod app;
mod device;
mod keycode_converter;
mod keycodes;
mod layout;

use std::error::Error;

use eframe::egui;

use keycode_converter::{Key, KeyExprBuilder as Expr, Modifier, step_expr};
use layout::Layout;

fn usage() {
    eprintln!("usage: jpad [gui|show|verify|demo]");
    eprintln!("  gui   open the layout editor (default)");
    eprintln!("  show  print the layout the device currently has in RAM");
    eprintln!("  verify  read, write back, read again and save to flash");
    eprintln!("  demo  send an example layout with the granular commands");
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1).as_deref() {
        None | Some("gui") => run_gui(),
        Some("show") => show(),
        Some("verify") => verify(),
        Some("demo") => demo(),
        Some(other) => {
            usage();
            eprintln!("unknown command: {other}");
            std::process::exit(2);
        }
    }
}

fn run_gui() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 720.0])
            .with_min_inner_size([860.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "J-Pad Keyboard Configuration",
        options,
        Box::new(|_cc| Ok(Box::new(app::JPadApp::default()))),
    )
    .map_err(|error| -> Box<dyn Error> { Box::new(std::io::Error::other(error.to_string())) })
}

fn demo() -> Result<(), Box<dyn Error>> {
    let mut device = device::Device::open()?;
    println!("connected to {}", device.description());

    let status = device.ping()?;
    println!("device reports {}", status.summary());

    device.set_macro(0, 4, &[step_expr(&[Expr::switch_layer(5)])?], 0)?;

    device.set_macro(
        5,
        5,
        &[
            step_expr(&[Expr::with_mod(Modifier::Lctl, Expr::key(Key::C))])?,
            step_expr(&[Expr::key(Key::Delete)])?,
        ],
        50,
    )?;

    device.set_macro(5, 6, &[step_expr(&[Expr::key(Key::Num7)])?], 0)?;
    device.set_repeat(5, 6, 100)?;

    device.set_encoder(
        5,
        step_expr(&[Expr::key(Key::B)])?[0],
        step_expr(&[Expr::key(Key::A)])?[0],
    )?;

    device.set_macro(5, 4, &[step_expr(&[Expr::switch_layer(0)])?], 0)?;

    device.save_layout()?;
    println!("layout saved in flash");

    let layout = device.read_layout()?;
    print_layer(&layout, 0);
    print_layer(&layout, 5);

    Ok(())
}

fn verify() -> Result<(), Box<dyn Error>> {
    let mut device = device::Device::open()?;
    let status = device.ping()?;
    println!("connected to {}", device.description());
    println!("device reports {}", status.summary());

    let before = device.read_layout()?;
    println!("read {} bytes", before.to_bytes().len());

    device.write_layout(&before)?;
    let after = device.read_layout()?;
    if before == after {
        println!("write and read back match");
    } else {
        println!("write and read back differ, the device changed the layout");
    }

    device.save_layout()?;
    let status = device.ping()?;
    println!("saved {} bytes to flash", status.stored_bytes);

    println!("unplug and replug the board, then run this again to see if it survived");
    Ok(())
}

fn show() -> Result<(), Box<dyn Error>> {
    let mut device = device::Device::open()?;
    let status = device.ping()?;
    println!("connected to {}", device.description());
    println!("device reports {}", status.summary());
    if status.unsaved_changes {
        println!("the device has layout changes that are not in flash");
    }

    let layout = device.read_layout()?;
    for layer in 0..layout.layers.len() {
        print_layer(&layout, layer);
    }

    Ok(())
}

fn print_layer(layout: &Layout, index: usize) {
    let Some(layer) = layout.layers.get(index) else {
        return;
    };

    println!("layer {index}");
    for (index, slot) in layer.macros.iter().enumerate() {
        let steps: Vec<String> = (0..slot.steps.len())
            .map(|step| slot.step_text(step))
            .filter(|text| !text.is_empty())
            .collect();

        if steps.is_empty() && slot.repeat_ms == 0 {
            continue;
        }

        let repeat = if slot.repeat_ms == 0 {
            "off".to_string()
        } else {
            format!("{}ms", slot.repeat_ms)
        };
        println!(
            "  K{:<2} {:<28} delay {:>4}ms repeat {repeat}",
            index + 1,
            steps.join(" then "),
            slot.delay_ms,
        );
    }

    println!(
        "  encoder ccw {} cw {}",
        keycode_converter::keycode_to_expr(layer.counterclockwise),
        keycode_converter::keycode_to_expr(layer.clockwise)
    );
}
