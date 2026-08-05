#[derive(Debug)]
pub struct StackMap {
    pub header: Header,
    pub num_functions: u32,
    pub num_constants: u32,
    pub num_records: u32,
    pub stack_size_records: Vec<StkSizeRecord>,
    pub large_constants: Vec<u64>,
    pub stack_map_records: Vec<StkMapRecord>,
}

#[derive(Debug)]
pub struct Header {
    pub version: u8,
}

#[derive(Debug)]
pub struct StkSizeRecord {
    pub function_addr: u64,
    pub stack_size: u64,
    pub record_count: u64,
}

#[derive(Debug)]
pub struct StkMapRecord {
    pub patchpoint_id: u64,
    pub instruction_offset: u32,
    pub locations: Vec<Location>,
    pub live_outs: Vec<LiveOut>,
}

#[derive(Debug)]
pub struct Location {
    pub size: u16,
    pub typ: LocationType,
}

#[derive(Debug)]
pub enum LocationType {
    Register(u16),
    Direct(u16, i32),
    Indirect(u16, i32),
    Constant(i32),
    ConstIndex(i32),
}

#[derive(Debug)]
pub struct LiveOut {
    pub reg_num: u16,
    pub size: u8,
}

impl From<&[u8]> for StackMap {
    fn from(bytes: &[u8]) -> Self {
        let mut parser = ByteParser::new(bytes);
        let header = parse_header(&mut parser);
        let num_functions = parser.get_u32();
        let num_constants = parser.get_u32();
        let num_records = parser.get_u32();
        let mut stack_size_records = vec![];
        let mut large_constants = vec![];
        let mut stack_map_records = vec![];

        for _ in 0..num_functions {
            stack_size_records.push(parse_stack_size_record(&mut parser));
        }

        for _ in 0..num_constants {
            large_constants.push(parser.get_u64());
        }

        for _ in 0..num_records {
            stack_map_records.push(parse_stack_map_record(&mut parser));
        }

        StackMap {
            header,
            num_functions,
            num_constants,
            num_records,
            stack_size_records,
            large_constants,
            stack_map_records,
        }
    }
}

fn parse_header(parser: &mut ByteParser) -> Header {
    let version = parser.get_u8();
    parser.get_u8();
    parser.get_u16();
    Header { version }
}

fn parse_stack_size_record(parser: &mut ByteParser) -> StkSizeRecord {
    StkSizeRecord {
        function_addr: parser.get_u64(),
        stack_size: parser.get_u64(),
        record_count: parser.get_u64(),
    }
}

fn parse_stack_map_record(parser: &mut ByteParser) -> StkMapRecord {
    let patchpoint_id = parser.get_u64();
    let instruction_offset = parser.get_u32();
    parser.get_u16();

    let num_locations = parser.get_u16();
    let mut locations = vec![];
    for _ in 0..num_locations {
        locations.push(parse_location(parser));
    }
    parser.align(8);

    parser.get_u16();

    let num_live_outs = parser.get_u16();
    let mut live_outs = vec![];
    for _ in 0..num_live_outs {
        live_outs.push(parse_live_out(parser));
    }
    parser.align(8);

    StkMapRecord {
        patchpoint_id,
        instruction_offset,
        locations,
        live_outs,
    }
}

fn parse_location(parser: &mut ByteParser) -> Location {
    let typ = parser.get_u8();
    parser.get_u8();
    let size = parser.get_u16();
    let reg_num = parser.get_u16();
    parser.get_u16();
    let offset = parser.get_i32();

    let typ = match typ {
        1 => LocationType::Register(reg_num),
        2 => LocationType::Direct(reg_num, offset),
        3 => LocationType::Indirect(reg_num, offset),
        4 => LocationType::Constant(offset),
        5 => LocationType::ConstIndex(offset),
        _ => panic!("Unexpected location type: {typ}"),
    };

    Location { size, typ }
}

fn parse_live_out(parser: &mut ByteParser) -> LiveOut {
    let reg_num = parser.get_u16();
    parser.get_u8();
    let size = parser.get_u8();

    LiveOut { size, reg_num }
}

struct ByteParser<'a> {
    bytes: &'a [u8],
    start: usize,
}

impl<'a> ByteParser<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        ByteParser { bytes, start: 0 }
    }

    fn get_u8(&mut self) -> u8 {
        let value = u8::from_le_bytes(self.bytes[self.start..self.start + 1].try_into().unwrap());
        self.start += 1;
        value
    }

    fn get_u16(&mut self) -> u16 {
        let value = u16::from_le_bytes(self.bytes[self.start..self.start + 2].try_into().unwrap());
        self.start += 2;
        value
    }

    fn get_u32(&mut self) -> u32 {
        let value = u32::from_le_bytes(self.bytes[self.start..self.start + 4].try_into().unwrap());
        self.start += 4;
        value
    }

    fn get_i32(&mut self) -> i32 {
        let value = i32::from_le_bytes(self.bytes[self.start..self.start + 4].try_into().unwrap());
        self.start += 4;
        value
    }

    fn get_u64(&mut self) -> u64 {
        let value = u64::from_le_bytes(self.bytes[self.start..self.start + 8].try_into().unwrap());
        self.start += 8;
        value
    }

    fn align(&mut self, alignment: usize) {
        if !self.start.is_multiple_of(alignment) {
            self.start += alignment - (self.start % alignment);
        }
    }
}
