#![no_std]

// UART

#[macro_export]
macro_rules! uart {
    ($(
        $UARTX:ident: $PACUARTX:ty,
    )+) => {
        $(
            pub struct $UARTX {
                pub registers: $PACUARTX,
            }

            impl embedded_hal::serial::Write<u8> for $UARTX {
                type Error = core::convert::Infallible;

                fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
                    // Wait until TXFULL is `0`
                    if self.registers.txfull.read().bits() != 0 {
                        Err(nb::Error::WouldBlock)
                    } else {
                        unsafe {
                            self.registers.rxtx.write(|w| w.rxtx().bits(word.into()));
                        }
                        Ok(())
                    }
                }
                fn flush(&mut self) -> nb::Result<(), Self::Error> {
                    if self.registers.txempty.read().bits() != 0 {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl embedded_hal::blocking::serial::write::Default<u8> for $UARTX {}
        )+
    }
}

// GPIO

#[macro_export]
macro_rules! gpio {
    ($(
        $GPIOX:ident: $PACGPIOX:ty,
    )+) => {
        $(
            pub struct $GPIOX {
                pub index: usize,
            }

            impl embedded_hal::digital::v2::OutputPin for $GPIOX {
                type Error = core::convert::Infallible;

                fn set_low(&mut self) -> Result<(), Self::Error> {
                    let reg = unsafe { &*<$PACGPIOX>::ptr() };
                    let mask: u32 = !(1 << self.index);
                    riscv::interrupt::free(|_cs| {
                        let val: u32 = reg.out.read().bits() & mask;
                        unsafe {
                            reg.out.write(|w| w.bits(val));
                        }
                    });
                    Ok(())
                }
                fn set_high(&mut self) -> Result<(), Self::Error> {
                    let reg = unsafe { &*<$PACGPIOX>::ptr() };
                    let mask: u32 = 1 << self.index;
                    riscv::interrupt::free(|_cs| {
                        let val: u32 = reg.out.read().bits() | mask;
                        unsafe {
                            reg.out.write(|w| w.bits(val));
                        }
                    });
                    Ok(())
                }
            }

            impl embedded_hal::digital::v2::StatefulOutputPin for $GPIOX {
                fn is_set_low(&self) -> Result<bool, Self::Error> {
                    let reg = unsafe { &*<$PACGPIOX>::ptr() };
                    let mask: u32 = 1 << self.index;
                    let val: u32 = reg.out.read().bits() & mask;
                    Ok(val == 0)
                }
                fn is_set_high(&self) -> Result<bool, Self::Error> {
                    let reg = unsafe { &*<$PACGPIOX>::ptr() };
                    let mask: u32 = 1 << self.index;
                    let val: u32 = reg.out.read().bits() & mask;
                    Ok(val != 0)
                }
            }

            /// Opt-in to the software implementation.
            impl embedded_hal::digital::v2::toggleable::Default for $GPIOX {}
        )+
    }
}

// SPI

#[macro_export]
macro_rules! spi {
    ($(
        $SPIX:ident: ($PACSPIX:ty, $WORD:ty),
    )+) => {
        $(
            pub struct $SPIX {
                pub registers: $PACSPIX,
            }

            impl embedded_hal::spi::FullDuplex<$WORD> for $SPIX {
                type Error = core::convert::Infallible;

                fn read(&mut self) -> nb::Result<$WORD, Self::Error> {
                    if self.registers.status.read().done().bit() {
                        Ok(self.registers.miso.read().bits() as $WORD)
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
                    if self.registers.status.read().done().bit() {
                        unsafe {
                            self.registers.mosi.write(|w| w.bits(word.into()));
                            self.registers.control.write(|w| {
                                w.length().bits(8).start().bit(true)
                            });
                        }
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }

            impl embedded_hal::blocking::spi::write::Default<u8> for $SPIX {}
            //impl embedded_hal::blocking::spi::write_iter::Default<u8> for $SPIX {}
            impl embedded_hal::blocking::spi::transfer::Default<u8> for $SPIX {}
        )+
    }
}

// Delay


#[macro_export]
macro_rules! timer {
    ($(
        $TIMERX:ident: $PACTIMERX:ty,
    )+) => {
        $(
            pub struct $TIMERX {
                pub registers: $PACTIMERX,
                pub sys_clk: u32,
            }

            impl<UXX: core::convert::Into<u32>> embedded_hal::blocking::delay::DelayMs<UXX> for $TIMERX {
                //type Error = Infallible;

                fn delay_ms(&mut self, ms: UXX) -> () {
                    let value: u32 = self.sys_clk / 1_000 * ms.into();
                    unsafe {
                        self.registers.en.write(|w| w.bits(0));
                        self.registers.reload.write(|w| w.bits(0));
                        self.registers.load.write(|w| w.bits(value));
                        self.registers.en.write(|w| w.bits(1));
                        self.registers.update_value.write(|w| w.bits(1));
                        while self.registers.value.read().bits() > 0 {
                            self.registers.update_value.write(|w| w.bits(1));
                        }
                    }
                }
            }
            impl<UXX: core::convert::Into<u32>> embedded_hal::blocking::delay::DelayUs<UXX> for $TIMERX {
                //type Error = Infallible;

                fn delay_us(&mut self, us: UXX) -> () {
                    let value: u32 = self.sys_clk / 1_000_000 * us.into();
                    unsafe {
                        self.registers.en.write(|w| w.bits(0));
                        self.registers.reload.write(|w| w.bits(0));
                        self.registers.load.write(|w| w.bits(value));
                        self.registers.en.write(|w| w.bits(1));
                        self.registers.update_value.write(|w| w.bits(1));
                        while self.registers.value.read().bits() > 0 {
                            self.registers.update_value.write(|w| w.bits(1));
                        }
                    }
                }
            }
        )+
    }
}
