pub fn maybe_is_code(n: i32) -> bool {
    let digits: Vec<u8> = number_digits(n);
    is_sorted(&digits) && !is_sort_unique(&digits)
}

fn number_digits(n: i32) -> Vec<u8> {
    let mut n_abs = u32::try_from(n.abs()).unwrap();
    let mut digits = Vec::new();
    while n_abs > 0 {
        digits.push(u8::try_from(n_abs % 10).unwrap());
        n_abs /= 10;
    }

    digits.reverse();
    digits
}

fn is_sorted<E>(s: &[E]) -> bool
where
    E: PartialOrd,
{
    s.len() < 2 || s.windows(2).all(|w| w[0] <= w[1])
}

fn is_sort_unique<E>(s: &[E]) -> bool
where
    E: Ord,
{
    assert!(is_sorted(s));
    s.len() < 2 || s.windows(2).all(|w| w[0] != w[1])
}
