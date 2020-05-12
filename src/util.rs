fn page_crossed(addr1: u16, addr2: u16) -> bool {
    addr1 & 0xff00 != addr2 & 0xff00
}

