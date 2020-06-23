use tokio::prelude::*;
use tokio::net::TcpStream;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::time::delay_for;
use std::time::Duration;
use std::f64::consts::PI;

fn read_f64(slice: &[u8]) -> f64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(slice);
    f64::from_be_bytes(bytes)
}

async fn ur_reader(mut r: ReadHalf<'_>) -> io::Result<()> {
    loop {
        let mut size_bytes = [0u8; 4];
        r.read_exact(&mut size_bytes).await?;
        let msg_size = u32::from_be_bytes(size_bytes) as usize;

        // need to subtract the 4 we already read
        let mut buf: Vec<u8> = Vec::new(); buf.resize(msg_size - 4, 0);
        r.read_exact(&mut buf).await?;

        if msg_size == 1116 {
            let time = read_f64(&buf[0..8]);
            println!("t: {:.1}s", time);
            for i in 0..6 {
                let index = 248+i*8;
                let joint_val = read_f64(&buf[index..index+8]);
                println!("q{}: {:>7.2}", i, 180.0/PI*joint_val);
            }
        }
    }
}

async fn ur_writer(mut w: WriteHalf<'_>) -> io::Result<()> {
    let mut on = false;
    loop {
        let message = format!("def myProg()\n  set_digital_out(0,{})\nend",
                              if on { "True" } else { "False" });
        w.write_all(message.as_bytes()).await?;
        delay_for(Duration::from_millis(500)).await;
        on = !on;
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("0.0.0.0:30003").await?;
    let (r, w) = stream.split();
    tokio::try_join!(ur_reader(r), ur_writer(w))?;
    Ok(())
}
