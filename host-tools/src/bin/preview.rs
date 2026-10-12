use dashboard_core::gfx::{Frame, BUFFER_SIZE};

fn main() -> std::io::Result<()> {
    let mut fb: Box<Frame> = Box::new([0u8; BUFFER_SIZE]);
    host_tools::render_sample(&mut fb);
    std::fs::write("preview.raw", &fb[..])?;
    println!("preview: wrote preview.raw");
    Ok(())
}
