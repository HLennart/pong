use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Disable interrupts during write to prevent deadlocks on Mutex<Writer>
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

impl From<u8> for Color {
    fn from(n: u8) -> Self {
        use Color::*;
        match n {
            0 => Black,
            1 => Blue,
            2 => Green,
            3 => Cyan,
            4 => Red,
            5 => Magenta,
            6 => Brown,
            7 => LightGray,
            8 => DarkGray,
            9 => LightBlue,
            10 => LightGreen,
            11 => LightCyan,
            12 => LightRed,
            13 => Pink,
            14 => Yellow,
            15 => White,
            _ => panic!("Invalid Color")
        }
    }
}

impl Color {
    pub fn next_color(&self) -> Color {
        let mut val = *self as u8;
        if val > 14 {
            val = 1;
        }
        val += 1;
        val.into()
    }

    pub fn previous_color(&self) -> Color {
        let mut val = *self as u8;
        if val < 2 {
            val = 15;
        }

        val -= 1;

        val.into()
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(pub u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

// There are 25 rows
pub const BUFFER_HEIGHT: usize = 25;

// There are 80 columns
pub const BUFFER_WIDTH: usize = 80;

// Make ScreenChar volatile so read/writes aren't
// optimized away
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col] = Volatile::new(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_byte_at_pos(&mut self, row: usize, col: usize, byte: u8) {
        let color_code = self.color_code;
        self.buffer.chars[row][col] = Volatile::new(ScreenChar {
            ascii_character: byte,
            color_code,
        });
    }

    pub fn write_screen_char_at_pos(&mut self, row: usize, col: usize, screen_char: ScreenChar) {
        self.buffer.chars[row][col] = Volatile::new(screen_char);
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable chars according to vga spec
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Non-printable chars according to vga spec
                _ => self.write_byte(0xfe),
            }
        }
    }

    #[allow(dead_code)]
    pub fn write_string_at_pos(&mut self, row: usize, mut col: usize, s: &str) {
        s.bytes().into_iter().for_each(|byte| {
            match byte {
            0x20..=0x7e => self.write_byte_at_pos(row, col, byte),
            _ => self.write_byte_at_pos(row, col, 0xfe),            
            };
            col += 1;
            if col > BUFFER_WIDTH - 1 {
                col = 0;
            }
        });

    
    }

    #[allow(dead_code)]
    pub fn write_bytes_at_pos(&mut self, row: usize, mut col: usize, bs: &[u8]) {
        bs.iter().for_each(|byte| {
            match byte {
                0x20..=0x7e => self.write_byte_at_pos(row, col, *byte),
                _ => self.write_byte_at_pos(row, col, 0xfe),
            }
            col += 1;
        });
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    #[allow(dead_code)]
    pub fn set_at_pos(&mut self, row: usize, col: usize, screen_char: ScreenChar) {
        self.buffer.chars[row][col].write(screen_char)
    }

    #[allow(dead_code)]
    pub fn clear_screen(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        self.buffer.chars.iter_mut().skip(1).for_each(|row| {
            row.iter_mut().for_each(|char| {
                char.write(blank);
            })
        });
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
