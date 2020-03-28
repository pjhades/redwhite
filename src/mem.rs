pub trait Access {
    // read a single byte
    fn read(&self, addr: u16) -> u8;

    // write a single byte
    fn write(&mut self, addr: u16, value: u8);

    // read 2 bytes starting from `addr`
    fn read16(&self, addr: u16) -> u16 {
        self.read(addr) as u16 | (self.read(addr + 1) as u16) << 8
    }

    // read 2 bytes but with lower address wrapped around
    // http://nesdev.com/6502_cpu.txt
    fn read16_wrapped(&self, addr: u16) -> u16 {
        let wrapped = addr & 0xff00 | (addr + 1) & 0x00ff;
        self.read(addr) as u16 | (self.read(wrapped) as u16) << 8
    }
}
