mod leet_code;
mod life;
mod matrices;
fn main() {
    let vec2 = [[0, 1, 2, 3], [4, 5, 6, 7]];
    let mut columnwise: Vec<Vec<i32>> = vec![];
    let mut i = 0;
    while i < vec2[0].len() {
        let mut column: Vec<i32> = vec![];
        for row2 in vec2 {
            column.push(row2[i]);
        }
        columnwise.push(column);
        i += 1;
    }
    println!("{:?}", columnwise);
}
// how to
