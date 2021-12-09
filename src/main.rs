
extern crate simple_allocator2;


use simple_allocator2::print_cost;
use simple_allocator2::ALLOCATOR;
use simple_allocator2::breakpoint;




fn test(_a:i32,_b:i32){
    let mut vector = vec![1, 2, 4, 8];// 16
    let mut vector2 = vec![9, 10, 11, 12];// 16
    
    vector2.push(13);
    // let mut a = vector[0];
    let mut a = vector2[4];
    // breakpoint(a as usize);
    {
        let mut vector3 = vec![1, 2, 4, 8];// 16
        let mut vector4 = vec![1, 2, 4, 8];// 16
    }
    // let _t = Vec::<u8>::with_capacity(127);
    // let _t = Vec::<u8>::with_capacity(64);
    // let _t = Vec::<u8>::with_capacity(64);
    
    // let mut vector2 = vec![1, 2, 4, 8];
    // // vector.push(16);
    // // vector.push(32);
    // // vector.push(64);
    // // vector.push(64);
    // {
    //     let mut v2: Vec<i32> = vec![8, 12, 14];
    // }
    // let mut v3: Vec<i32> = vec![16, 32, 64];
    // vector.append(&mut v3);

}


fn main() {
    
    // println!("{}",mem::size_of::<MyPage>());
    let cc = 4;
    unsafe{
        print_cost!(test(2+2,cc));
    
    }
    // println!("{}", count.temp.get());
}
