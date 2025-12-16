#![no_std]

use syscalls::{
    syscall,
    Sysno
};

const MAX_BUFFER_PRINTED: usize = 43;
enum Res<T, E> {
    Ok(T),
    Err(E)
}

impl<T, E> Res<T, E> {
    fn is_ok(&self) -> bool {
        if let Res::Ok(_) = self {
            return true
        }
        return false
    }
    fn unwrap(self) -> T {
        if let Res::Ok(thing) = self {
            return thing
        }
        panic!("Res Returned an ERR value")
    }
}
trait PrintAble {
    fn to_bytes(&self) -> [u8; MAX_BUFFER_PRINTED] {[0u8; MAX_BUFFER_PRINTED]} //Default implementation.
}
impl PrintAble for u64 {
    fn to_bytes(&self) -> [u8; MAX_BUFFER_PRINTED] {
        let mut buf: [u8; MAX_BUFFER_PRINTED] = [0x0; MAX_BUFFER_PRINTED];
        let mut n = *self;
        if n == 0 {
            buf[0] = b'0';
            return buf;
        }
        let mut len = 0;
        while n > 0 && len < MAX_BUFFER_PRINTED {
            buf[len] = b'0' + (n%10) as u8;
            n /=10;
            len += 1;
        }

        let mut i = 0;
        while i < len/2 { 
            let j = len - 1 -i; 
            let tmp = buf[i];
            buf[i] = buf[j];
            buf [j] = tmp;
            i+=1
        }
        buf
    }
}
impl PrintAble for &str {
    fn to_bytes(&self) -> [u8; MAX_BUFFER_PRINTED] {
        let mut buf: [u8; MAX_BUFFER_PRINTED] = [0x0; MAX_BUFFER_PRINTED];
        let bytes = self.as_bytes();
        let len: usize = bytes.len();
        // unsafe {
        //     syscall!(Sysno::write, bytes[..MAX_BUFFER_PRINTED].as_ptr(), buf.as_mut_ptr(), bytes.len()).unwrap();
        // }
        buf[..len].copy_from_slice(&bytes);
        return buf;
    }
}



//Impliment read_line and write_to_line
fn print<T: PrintAble>(string: T) -> Res<(), syscalls::Errno> {
    const O_WRONLY: u8 = 0o1;
    let tty = b"/dev/tty\0";
    let fd = unsafe {
        syscall!(Sysno::open, tty.as_ptr(), O_WRONLY) //Open /dev/tty in Write mode
    }.unwrap();
    //Now we write to the tty.
    let buffer = string.to_bytes();
    unsafe {
        syscall!(Sysno::write, fd, buffer.as_ptr(), buffer.len()).unwrap()
    };
    Res::Ok(())
}

fn exit(code: u8) -> ! {
    unsafe {
        let _ = syscall!(Sysno::exit, code);
        core::hint::unreachable_unchecked();
    }
}

fn read() -> Res<u64, syscalls::Errno> {
    const O_RDONLY: u8 = 0o0;
    let tty = b"/dev/tty\0";
    let fd = unsafe {
        syscall!(Sysno::open, tty.as_ptr(), O_RDONLY) //Open /dev/tty in Read-only mode
    }.unwrap();

    let mut buff: [u8; 256] = [0u8; 256];
    let bytes_read = unsafe {
        syscall!(Sysno::read, fd, buff.as_mut_ptr(), buff.len()).unwrap()
    };
    let mut bytes_recieved = buff[..bytes_read as usize].to_vec();
    if let Some(pos) = bytes_recieved.iter().position(|byte| *byte == b'\n') {
        bytes_recieved.truncate(pos);
    }
    let string = match core::str::from_utf8(&bytes_recieved) {
        Ok(s) => s,
        Err(_) => {let _ = print("You did not enter a valid utf8 string."); exit(1)}
    };
    //Now parsing the string.
    let number: u64 = match string.parse() { //No need to trim cause we truncated earlier.
        Ok(num) => num,
        Err(_) => { let _ = print("Please enter a valid Number."); exit(1);}
    };
    Res::Ok(number)
}


macro_rules! square {
    ($x: expr) => {{
        $x * $x
    }};
}

fn call_macro(var: u64) -> () {
    let _ = print(var);
    let _ = print(" * ");
    let _ = print(var);
    let _ = print(" = ");
    let _ = print(square!(var));
}

fn main() -> () {
    // print_expr!(1+1);
    let _ = print("We are now going to play a squaring game.\n\0");
    let _ = print("Enter a number: \0");
    let number = read();
    if number.is_ok() {
        call_macro(number.unwrap())
    }
}