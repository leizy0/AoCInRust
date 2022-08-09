use std::collections::HashSet;

fn main() {
    let mut reg1: usize = 0;
    let mut ind = 0;
    let mut reg0_set = HashSet::new();
    let mut reg0s = Vec::new();
    loop {
        let mut reg2: usize = reg1 | 0x10000;
        reg1 = 6663054;
        loop {
            reg1 += reg2 & 0xFF;
            reg1 &= 0xFFFFFF;
            reg1 *= 65899;
            reg1 &= 0xFFFFFF;
            if reg2 < 256 {
                break;
            }
            let mut reg3 = 1;
            while reg3 * 256 <= reg2 {
                reg3 += 1;
            }
            reg2 = reg3 - 1;
        }

        println!("Loop#{}: register[1] = {}", ind, reg1);
        if reg0_set.contains(&reg1) {
            break;
        } else {
            reg0_set.insert(reg1);
            reg0s.push(reg1);
        }
        ind += 1;
    }

    print!("Set register 0 to {}, can halt program and cost the most steps", reg0s.last().unwrap());
}