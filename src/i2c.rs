//! I2C interfaces

use core::convert::Infallible;
use core::fmt::Debug;
use core::task::Poll;

use embedded_hal::blocking::i2c::{Operation, Read, Transactional, Write};
use embedded_hal::digital::v2::InputPin;

use crate::{AsyncInterface, Interface};

/// To be used in `Interface::wait_ready` implementations
pub const PN532_I2C_READY: u8 = 0x01;

/// I2C address of the Pn532
pub const I2C_ADDRESS: u8 = 0x24;

/// I2C Interface without IRQ pin
#[derive(Clone, Debug)]
pub struct I2CInterface<I2C>
where
    I2C: Transactional,
    I2C: Write<Error = <I2C as Transactional>::Error>,
    I2C: Read<Error = <I2C as Transactional>::Error>,
    <I2C as Transactional>::Error: Debug,
{
    pub i2c: I2C,
}

impl<I2C> Interface for I2CInterface<I2C>
where
    I2C: Transactional,
    I2C: Write<Error = <I2C as Transactional>::Error>,
    I2C: Read<Error = <I2C as Transactional>::Error>,
    <I2C as Transactional>::Error: Debug,
{
    type Error = <I2C as Transactional>::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(I2C_ADDRESS, frame)
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        let mut buf = [0];
        self.i2c.read(I2C_ADDRESS, &mut buf)?;

        if buf[0] == PN532_I2C_READY {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.exec(
            I2C_ADDRESS,
            &mut [Operation::Read(&mut [0]), Operation::Read(buf)],
        )
    }
}

/// I2C Interface with IRQ pin
#[derive(Clone, Debug)]
pub struct I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: Transactional,
    I2C: Write<Error = <I2C as Transactional>::Error>,
    I2C: Read<Error = <I2C as Transactional>::Error>,
    <I2C as Transactional>::Error: Debug,
    IRQ: InputPin<Error = Infallible>,
{
    pub i2c: I2C,
    pub irq: IRQ,
}

impl<I2C, IRQ> Interface for I2CInterfaceWithIrq<I2C, IRQ>
where
    I2C: Transactional,
    I2C: Write<Error = <I2C as Transactional>::Error>,
    I2C: Read<Error = <I2C as Transactional>::Error>,
    <I2C as Transactional>::Error: Debug,
    IRQ: InputPin<Error = Infallible>,
{
    type Error = <I2C as Transactional>::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(I2C_ADDRESS, frame)
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        // infallible unwrap because of IRQ bound
        if self.irq.is_low().unwrap() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.exec(
            I2C_ADDRESS,
            &mut [Operation::Read(&mut [0]), Operation::Read(buf)],
        )
    }
}

pub struct AsyncI2cInterface<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    pub i2c: I2C,
}

impl<I2C> AsyncInterface for AsyncI2cInterface<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    type Error = I2C::Error;

    async fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(I2C_ADDRESS, frame).await
    }

    async fn wait_ready(&mut self) -> Result<(), Self::Error> {
        let mut buf = [0];

        loop {
            self.i2c.read(I2C_ADDRESS, &mut buf).await?;
            if buf[0] == PN532_I2C_READY {
                return Ok(());
            }
        }
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.read(I2C_ADDRESS, buf).await
    }
}
