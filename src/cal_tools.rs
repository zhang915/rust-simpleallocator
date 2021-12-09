// 计算工具 用来做页表头的位运算
const power_2: &[u64] = &[4, 16, 256, 65536, 4294967296 ];
const BLOCK_SIZES: &[usize] = &[16, 32, 64, 128, 256, 512, 1024, 2048];

// 把2^i次方转化成i
pub fn power_int(j: u64) -> usize {
    let mut a = j;
    let mut i: usize = 0;
    while a != 0 {
        a = a >> 1;
        i = i + 1;
    }
    i - 1
}

// 在deallocate时计算，把指针下标转化成具体位置，更改freelist的具体位置
// 找到对应的block常量的arr pos 和 arr[pos] 应当减去的大小
// i是ptr的偏移量 0<=i<256 size_class是page大小
pub fn decal_position(i :usize, size_class: usize) -> (usize, u64){
    let mut pos: usize = 0;
    // 先获取对应的pos
    pos = i / 64 ;
    let mut j: usize = i;
    j = j - pos * 64;
    let mut ret = 1;
    // 返回一个需要减去的大小
    if j == 0{
        return (pos, 1);
    }
    while j >= 1 {
        j = j - 1;
        ret = ret * 2;
    }
    (pos, ret)
}

// 返回一个u64 表示这个项要存留的位置，直接返回经过计算后的ptr
// 如果是Err()表示此页已经被完全占有
// 需要计算size_class来保证Err的正确返回
pub fn cal_position(arr: &[u64;4], size_class: usize) -> Result<(usize, u64), ()> { //前面是角标，后面是大小
    // 如果size_class < 64 会使用超过arr第一个元素
    // 如果size_class >64 则只会使用arr第一个元素
    if size_class < 64 {
        let mut id: usize = 0;
        loop {
            let mut i = arr[id];
            let mut j: u64 = 1;
            loop {
                if i & 1 == 1{
                    i = i >> 1;
                    j = j << 1;
                }
                else {
                    return Ok((id,j));
                }
                if j == 0 {
                    if size_class == 16{
                        if id == 3 {
                            return Err(());
                        }
                        break;
                    }
                    if size_class == 32{
                        if id == 1 {
                            return Err(());
                        }
                        break;
                    }
                }
            }
            id = id + 1;
        }
    }else {
        // 如果size_class > 64 那么只有第一个元素会被使用，需要加入对于位运算的特殊判定
        let mut i = arr[0];
        let mut j: u64 = 1;
        loop {
            if i & 1 == 1{
                i = i >> 1;
                j = j << 1;
            }
            else {
                return Ok((0,j));
            }
            if j == 0 {// size_class=64
                return Err(());
            }
            if size_class == 128 {
                if j == power_2[4] {
                    return Err(());
                }
            }
            if size_class == 256 {
                if j == power_2[3] {
                    return Err(());
                }
            }
            if size_class == 512 {
                if j == power_2[2] {
                    return Err(());
                }
            }
            if size_class == 1024 {
                if j == power_2[1] {
                    return Err(());
                }
            }
            if size_class == 2048 {
                if j == power_2[0] {
                    return Err(());
                }
            }
        }
    }
}
// 用来返回block适应大小的下角标 i就是blocksize大小
pub fn find_block(i: usize) -> usize{

    if i > 2048 {
        // 如果是大文件，直接返回大文件链表下角标
        return 4;
    }

    let mut j: usize = 0;
    // --没有判断大小过大的情况,就是如果是大文件是判断不了的
    // --需要判断吗？
    loop {
        if BLOCK_SIZES[j] >= i{
            return j;
        }else {
            j = j + 1;
        }
    }
}
