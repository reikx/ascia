pub static CHARMAP3X3: [char; 256] = charmap_3x3();

pub const fn charmap_3x3() -> [char;256]{
    let mut map: [char;256] = ['_';256];

    let mut map_base: [char;256] = ['\0';256];
    map_base[0b01011111] = 'A';
    map_base[0b11111111] = '#';
    map_base[0b00000111] = '_';
    map_base[0b00000010] = '.';
    map_base[0b10010011] = '\\';
    map_base[0b11001001] = '\\';
    map_base[0b01110100] = '/';
    map_base[0b00101110] = '/';
    map_base[0b01000010] = '!';
    map_base[0b00011000] = '-';
    map_base[0b00000000] = ' ';
    map_base[0b11100000] = '"';
    map_base[0b00010111] = 'u';
    map_base[0b00010101] = 'x';
    map_base[0b01011111] = 'A';
    map_base[0b01011111] = 'A';
    map_base[0b01011111] = 'A';

    let mut i = 0u32;
    while i <= 255u32{
        let mut j = 0u32;
        let mut mem:(u32, char) = (u32::MAX, ' ');
        while j <= 255u32{
            if map_base[j as usize] == '\0'{
                j += 1;
                continue;
            }
            let distance = (i ^ j).count_ones();
            if distance < mem.0{
                mem.0 = distance;
                mem.1 = map_base[j as usize];
            }
            j = j + 1;
        }
        map[i as usize] = mem.1;
        i += 1;
    }
    return map;
}

