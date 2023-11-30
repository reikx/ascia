use std::collections::{HashMap};

pub struct CharMapper3x3{
    data:HashMap<u8,char>,
    pub mem:Vec<char>
}

impl CharMapper3x3 {
    pub fn new() -> Self{
        let mut cm = CharMapper3x3 {
            data:HashMap::new(),
            mem:vec![' ';256]
        };

        cm.data.insert(0b01011111,'A');
        cm.data.insert(0b11111111,'#');
        cm.data.insert(0b00000111,'_');
        cm.data.insert(0b00000010,'.');
        cm.data.insert(0b10010011,'\\');
        cm.data.insert(0b11001001,'\\');
        cm.data.insert(0b01110100,'/');
        cm.data.insert(0b00101110,'/');
        cm.data.insert(0b01000010,'!');
        cm.data.insert(0b00011000,'-');
        cm.data.insert(0b00000000,' ');
        cm.data.insert(0b11100000,'"');
        cm.data.insert(0b00010111,'u');
        cm.data.insert(0b00010101,'x');

        let mut scores:Vec<u32> = vec![u32::MAX;256];

        for seg in 0..=255{
            for c in &cm.data{
                let e = CharMapper3x3::eval((c.0).clone(), seg);
                if e < scores[seg as usize]{
                    scores[seg as usize] = e;
                    cm.mem[seg as usize] = c.1.clone();
                }
            }
        }
        return cm;
    }

    fn eval(seg_target:u8,seg_comp:u8) -> u32{
        let mut diff = 0;
        for i in 0..8 {
            let dist = CharMapper3x3::dist(seg_target, seg_comp, i);
           // println!("{}",dist);
            diff += (dist * dist) as u32;
        }
        return diff;
    }

    fn dist(seg_target:u8, seg_comp:u8, pos:u8) -> u8{
        /*
        println!("pos = {}",pos);
        println!("target -> \n{}",CharMatcher::seg_stringify(seg_target));
        println!("comp -> \n{}",CharMatcher::seg_stringify(seg_comp));
        */
        if pos == 0{
            if ((seg_comp >> 7) ^ (seg_target >> (7 - pos))) & 1 == 0{
                return 0;
            }
            else if ((seg_comp >> 6) ^ (seg_target >> (7 - pos))) & 1 == 0 || ((seg_comp >> 4) ^ (seg_target >> (7 - pos))) & 1 == 0{
                return 1;
            }
            else if ((seg_comp >> 5) ^ (seg_target >> (7 - pos))) & 1 == 0 {
                return 2;
            }
            else if ((seg_comp >> 3) ^ (seg_target >> (7 - pos))) & 1 == 0 || ((seg_comp >> 1) ^ (seg_target >> (7 - pos))) & 1 == 0{
                return 3;
            }
            else{
                return 4;
            }
        }
        else if pos == 1{
            if ((seg_comp >> 6) ^ (seg_target >> (7 - pos))) & 1 == 0{
                return 0;
            }
            else if ((seg_comp >> 7) ^ (seg_target >> (7 - pos))) & 1 == 0 || ((seg_comp >> 5) ^ (seg_target >> (7 - pos))) & 1 == 0{
                return 1;
            }
            else if ((seg_comp >> 4) ^ (seg_target >> (7 - pos))) & 1 == 0 || ((seg_comp >> 3) ^ (seg_target >> (7 - pos))) & 1 == 0{
                return 2;
            }
            else if ((seg_comp >> 2) ^ (seg_target >> (7 - pos))) & 1 == 0 || ((seg_comp >> 0) ^ (seg_target >> (7 - pos))) & 1 == 0{
                return 3;
            }
            else{
                return 4;
            }
        }
        else{
            let mut new_s_target = seg_target;
            let mut new_s_comp = seg_comp;

            //flip
            if pos == 2{
                new_s_target = CharMapper3x3::swap(new_s_target, 2, 0);
                new_s_target = CharMapper3x3::swap(new_s_target, 4, 3);
                new_s_target = CharMapper3x3::swap(new_s_target, 7, 5);

                new_s_comp = CharMapper3x3::swap(new_s_comp, 2, 0);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 4, 3);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 7, 5);

            }
            else if pos == 3{
                new_s_target = CharMapper3x3::swap(new_s_target, 3, 1);
                new_s_target = CharMapper3x3::swap(new_s_target, 5, 2);
                new_s_target = CharMapper3x3::swap(new_s_target, 6, 4);

                new_s_comp = CharMapper3x3::swap(new_s_comp, 3, 1);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 5, 2);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 6, 4);
            }
            else if pos == 4{
                new_s_target = CharMapper3x3::swap(new_s_target, 4, 1);
                new_s_target = CharMapper3x3::swap(new_s_target, 6, 3);
                new_s_target = CharMapper3x3::swap(new_s_target, 7, 0);

                new_s_comp = CharMapper3x3::swap(new_s_comp, 4, 1);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 6, 3);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 7, 0);
            }
            else if pos == 5{
                new_s_target = CharMapper3x3::swap(new_s_target, 5, 0);
                new_s_target = CharMapper3x3::swap(new_s_target, 6, 1);
                new_s_target = CharMapper3x3::swap(new_s_target, 7, 2);

                new_s_comp = CharMapper3x3::swap(new_s_comp, 5, 0);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 6, 1);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 7, 2);
            }
            else if pos == 6{
                new_s_target = CharMapper3x3::swap(new_s_target, 5, 0);
                new_s_target = CharMapper3x3::swap(new_s_target, 6, 1);
                new_s_target = CharMapper3x3::swap(new_s_target, 7, 2);

                new_s_comp = CharMapper3x3::swap(new_s_comp, 5, 0);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 6, 1);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 7, 2);
            }
            else if pos == 7{
                new_s_target = CharMapper3x3::swap(new_s_target, 7, 0);
                new_s_target = CharMapper3x3::swap(new_s_target, 6, 3);
                new_s_target = CharMapper3x3::swap(new_s_target, 4, 1);

                new_s_comp = CharMapper3x3::swap(new_s_comp, 7, 0);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 6, 3);
                new_s_comp = CharMapper3x3::swap(new_s_comp, 4, 1);
            }
            if pos == 2 || pos == 5 || pos == 7{
                return CharMapper3x3::dist(new_s_target, new_s_comp, 0);
            }
            else{
                return CharMapper3x3::dist(new_s_target, new_s_comp, 1);
            }
        }
    }

    fn swap(u:u8,n:u8,m:u8) -> u8{
        if n == m{
            return u;
        }
        let b1 = u & (1 << (7 - n));
        let b2 = u & (1 << (7 - m));
        if n < m{
            return u - b1 - b2 + (b1 >> (m - n)) + (b2 << (m - n));
        }
        else{
            return u - b1 - b2 + (b1 << (n - m)) + (b2 >> (n - m));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ascia::charmapper::CharMapper3x3;

    #[test]
    fn test() {
        let cm = CharMapper3x3::new();
        println!("{}", CharMapper3x3::eval(0b11100111, 0b00011000));
    }
}


