pub fn maybe_is_code(n: i32) -> bool {
    let digits: Vec<u8> = number_digits(n);
    is_sorted(&digits) && !sorted_is_unique(&digits)
}

pub fn maybe_is_code2(n: i32) -> bool {
    let digits: Vec<u8> = number_digits(n);
    is_sorted(&digits) && same_neighbor_counts(&digits).contains(&2)
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

fn sorted_is_unique<E>(s: &[E]) -> bool
where
    E: Ord,
{
    assert!(is_sorted(s));
    s.len() < 2 || s.windows(2).all(|w| w[0] != w[1])
}

fn same_neighbor_counts<E>(s: &[E]) -> Vec<usize>
where
    E: PartialEq,
{
    if s.is_empty() {
        return Vec::new();
    }

    if s.len() < 2 {
        return vec![1];
    }

    let mut cur_same_count = 1usize;
    let mut same_counts = Vec::new();
    let mut last_digit = &s[0];
    for e in &s[1..] {
        if e == last_digit {
            cur_same_count += 1;
        } else {
            same_counts.push(cur_same_count);
            cur_same_count = 1;
        }

        last_digit = e;
    }

    same_counts.push(cur_same_count);
    same_counts
}

#[test]
fn test_same_neighbor_counts_margin_value() {
    assert_eq!(same_neighbor_counts::<u8>(&[]), vec![]);
    assert_eq!(same_neighbor_counts(&[1]), vec![1]);
}

#[test]
fn test_same_neighbor_counts_unique() {
    assert_eq!(
        same_neighbor_counts(&[1, 2, 3, 7, 8, 9]),
        vec![1, 1, 1, 1, 1, 1]
    );
}

#[test]
fn test_same_neighbor_counts_same_neighbors() {
    assert_eq!(
        same_neighbor_counts(&[2, 2, 3, 4, 5, 0]),
        vec![2, 1, 1, 1, 1]
    );
    assert_eq!(same_neighbor_counts(&[1, 1, 2, 2, 3, 3]), vec![2, 2, 2]);
    assert_eq!(same_neighbor_counts(&[1, 1, 1, 1, 2, 2]), vec![4, 2]);
}
