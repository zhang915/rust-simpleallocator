#![feature(const_mut_refs)]
#![feature(cell_update)]
#![feature(as_array_of_cells)]
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ptr::null_mut;
use std::mem;
use std::borrow::BorrowMut;

pub mod cal_tools;

const MAX_SUPPORTED_ALIGN: usize = 4096;

use cal_tools::power_int;
use cal_tools::decal_position;
use cal_tools::cal_position;
use cal_tools::find_block;

pub struct Count{
    allocated: Cell<usize>,
    deallocated: Cell<usize>,
    peak: Cell<usize>,
    flag: Cell<bool>,
    alloc_msg: Cell<[(usize, usize); 4096]>,
    alloc_msg_point: Cell<usize>,
    // 测试数据
    temp: Cell<u64>,
}


pub struct PageListHead{
    
    ptr: *mut MyPage,
    // 表示此页项每一项的大小
    size_class: usize,
 
}

// Mypage 大小24B
pub struct MyPage{
    // 结构体中 3*8B
    pub next: *mut MyPage,
    pub free_list: [u64;4],
    pub size_class: usize,
}


impl PageListHead{
    const fn new(size_class:usize) -> PageListHead{
        PageListHead {
            ptr: 0 as *mut MyPage,
            size_class: size_class,

        }
    }
    // 看这个页的后面页项大小是多少
    fn get_sizeclass(&self) -> usize{
        self.size_class
    } 


    unsafe fn reserve_page(&self) -> *mut u8{
        let mut temp_ptr = self.ptr;
        
        let layout = Layout::from_size_align(4096 + mem::size_of::<MyPage>(), 1).unwrap();
        // default allocate 一块内存给它 并且采用存储方式是
        
        let memory_pool = ALLOCATOR.default_allocator.alloc(layout);
        // 如果不是0点，就一直向下找新开一页
        if temp_ptr != 0 as *mut MyPage {
            
            let mut current = temp_ptr as *mut MyPage;
            while (*current).next != 0 as *mut MyPage{
                current = (*current).next as *mut MyPage;
            }
            (*current).next = memory_pool as *mut MyPage;
        }
        // 如果是0点，直接新开一页，让self.ptr变成页起始地址
        else{
            (*(self as *const PageListHead as *mut PageListHead)).ptr = memory_pool as *mut MyPage;
        }
        // 初始化一个MyPage
        let new_page = memory_pool as *mut MyPage;
        (*new_page).next = 0 as *mut MyPage;
        (*new_page).free_list = [0, 0, 0, 0];
        (*new_page).size_class = self.size_class;
        // 返回这一页起始地址
        return memory_pool;
    }
    // 返回一个大小超过2048的页，这个页MyPage freelist不需要使用，但是会造成16byte损耗
    // -- 考虑碎片问题 小文件内存重用问题？
    unsafe fn reserve_large_page(&self, page_size: usize) -> *mut u8 {
        let temp_ptr = self.ptr;
        // layout 大小为 mypage页项大小 + 大文件本身大小
        let layout = Layout::from_size_align(page_size + mem::size_of::<MyPage>(), 1).unwrap();
        // 操作和之前基本一样
        let memory_pool = ALLOCATOR.default_allocator.alloc(layout);
        if temp_ptr != 0 as *mut MyPage {
            
            let mut current = temp_ptr as *mut MyPage;
            while (*current).next != 0 as *mut MyPage{
                current = (*current).next as *mut MyPage;
            }
            (*current).next = memory_pool as *mut MyPage;
        }
        // 如果是0点，直接新开一页，让self.ptr变成页起始地址
        else{
            // 一个离谱的转换方式，为了改变一个静态变量，将引用转化为 *const 再转*mut
            (*(self as *const PageListHead as *mut PageListHead)).ptr = memory_pool as *mut MyPage;
        }
        // 初始化一个MyPage
        let new_page = memory_pool as *mut MyPage;
        (*new_page).next = 0 as *mut MyPage;
        (*new_page).free_list = [0, 0, 0, 0];
        (*new_page).size_class = page_size;
        // 返回这一页起始地址
        return memory_pool;
    }


}



#[repr(C, align(4096))]
pub struct SimpleAllocator<DefaultAllocator: 'static + GlobalAlloc> {
    default_allocator: &'static DefaultAllocator,
}

thread_local!{
    pub static count: Count = Count{
        allocated: Cell::new(0),
        deallocated: Cell::new(0),
        peak: Cell::new(0),
        flag: Cell::new(false),
        alloc_msg: Cell::new([(0, 0); 4096]),
        alloc_msg_point: Cell::new(0),

        temp: Cell::new(0),
    };
    // 大文件的pagelist表的sizeclass是不相同的，大小只与大文件本身大小有关
    pub static MyPages:[PageListHead;5] = [PageListHead::new(16), PageListHead::new(32), PageListHead::new(64), PageListHead::new(128), PageListHead::new(2049)];
}

#[global_allocator]
pub static ALLOCATOR: SimpleAllocator<System> = SimpleAllocator {
    default_allocator: &System,
};


impl Count{
    pub fn init(&self){
        self.allocated.set(0);
        self.deallocated.set(0);
        self.peak.set(0);
        self.set_flag(true);
    }

    fn set_flag(&self, status: bool){
        self.flag.set(status);
    }

    pub fn get_flag(&self) -> bool{
        return self.flag.get();
    }

    pub fn record_msg(&self, size: usize, flag: usize){
        if self.alloc_msg_point.get() >= 4096{
            return;
        }
        let temp = self.alloc_msg.as_array_of_cells();
        temp[self.alloc_msg_point.get()].set((flag, size));
        self.alloc_msg_point.set(self.alloc_msg_point.get() + 1);
    }
    // 用来测试
    pub fn set_temp(&self, i:u64) {
        self.temp.set(i);
    }
    // 记录alloc信息
    pub fn finish(&self){
        self.set_flag(false);
        println!();
        println!("allocated:{:?}",self.allocated.get());
        println!("deallocated:{:?}",self.deallocated.get());
        println!("remaining:{:?}",self.allocated.get() as i32 - self.deallocated.get() as i32);
        println!("peak:{:?}",self.peak.get());
        println!();
        println!("temp:{:?}",self.temp.get());

        // for i in 0..self.alloc_msg_point.get(){
        //     print!("{:?};",self.alloc_msg.get()[i]);
        // }
        self.alloc_msg_point.set(0);
        println!();
    }
}


impl<DefaultAllocator: 'static + GlobalAlloc> SimpleAllocator<DefaultAllocator>{
    // 这里在初始化的时候对每个大小的都创建一个？
    pub unsafe fn init(&self){
        count.with(|f| {
            f.init();
        });

        // MyPages.with(|f|{
        //     // --
        //     f[0].reserve_page();
        //     println!("{:?}",(*(f[0].ptr as *mut MyPage)).size_class);
        // });
    }

    pub fn get_flag(&self) -> bool{
        return count.with(|f| {
                f.get_flag()
        });
    }

    pub fn record_msg(&self, size: usize, flag: usize){
        count.with(|f| {
                f.record_msg(size,flag);
        });
    }

    pub fn update_alloc(&self, size: usize){
        count.with(|f| {
            f.allocated.set(f.allocated.get() + size);
            if (f.allocated.get() as i32 - f.deallocated.get() as i32) > f.peak.get() as i32{
                f.peak.set(f.allocated.get() - f.deallocated.get());
            }
        });
    }

    pub fn update_dealloc(&self, size: usize){
        count.with(|f| {
                f.deallocated.set(f.deallocated.get() + size);
        });
    }

    pub fn finish(&self){
        count.with(|f| {
                f.finish();
        });
    }

}
// 打断点 用来debug
pub fn breakpoint(i: usize){
    count.with(|f|{
        f.set_flag(false);
    });
    println!("{}",i);
    count.with(|f|{
        f.set_flag(true);
    });
}

unsafe impl<DefaultAllocator: 'static + GlobalAlloc> Sync for SimpleAllocator<DefaultAllocator> {}

unsafe impl<DefaultAllocator: 'static + GlobalAlloc> GlobalAlloc for SimpleAllocator<DefaultAllocator> {
    // alloc最终返回的指针是在页中我们为它分配的页项的指针
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.get_flag() == false{
            return self.default_allocator.alloc(layout);
        }
        // 处理layout大小，判断应该在哪一个大小申请一页
        // 

        
        let i = find_block(layout.size());
        let mut ret_ptr: *mut u8 = 0 as *mut u8;
        // 特殊判定大文件
        // 大文件只需要新加页即可 在dealloc时对页直接进行
        if i == 4 {
            MyPages.with(|f|{
                ret_ptr = f[i].reserve_large_page(layout.size());
            });
            self.update_alloc(layout.size());
            self.record_msg(layout.size(), 1);
            return ret_ptr.add(mem::size_of::<MyPage>());
        }
        // test
        
        
        // 获得i之后，从第i个位置向后接页
        // 先判断是否需要进行加页
        MyPages.with(|f|{
            // 如果计算可行就直接插入而不新建page
            // 如果还没有第一个页 直接插入页
            // current 表示当前这个headlist的第一个mypage的ptr
            let mut current = f[i].ptr;
            loop {
                //在loop中遍历current到最后面，但是page中间如果有空页项可以直接插入
                //如果直接是0，就新开一页
                if current == 0 as *mut MyPage{
                    ret_ptr = f[i].reserve_page();
                    current = f[i].ptr;
                } 
                
                if let Ok((pos,t)) = cal_position(&(*(current as *mut MyPage)).free_list, (*(current as *mut MyPage)).size_class) {
                    
                    // 成功找到空页项
                    (*(current as *mut MyPage)).free_list[pos] = (*(current as *mut MyPage)).free_list[pos] + t;
                    // 放到位置pos*1024+t这里
                    ret_ptr = current as *mut u8;
                    ret_ptr = ret_ptr.add( pos * 1024 + power_int(t) * (*(current as *mut MyPage)).size_class );
                    // breakpoint((power_int(t) * (*(current as *mut MyPage)).size_class) as usize);
                    break ;
                }
                else {// 如果当前页满，current就向下遍历

                    current = (*current).next as *mut MyPage;
                }
            }
            
            //println!("{:?}",(*(f[i].ptr as *mut MyPage)).size_class);
        });    

        // --
        self.update_alloc(layout.size());
        self.record_msg(layout.size(), 1);
        return ret_ptr.add(mem::size_of::<MyPage>());
        //return System.alloc(layout);  
    }
    // 两个参数，分别是指针和layout，指针是alloc时的指针，layout是要释放的大小
    // dealloc要找到具体的页项，去掉页项标注而不对它进行操作，关键是如何找到对应的页标注
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        //if layout.size() != 1024{
          //  println!("{:?}", layout.size());
        //}
        if self.get_flag() == false{
            self.default_allocator.dealloc(ptr,layout);
            return;
        }

        // 对于ptr 需要先找到对应的page, 从page中获取ptr在freelist中的具体位置
        // 获得 layout 大小所对应的blocksize下标 i
        let i = find_block(layout.size());

        // 区分，如果是大文件的情况，需要额外的一个指针专门记录前一个块
        if i == 4{
            MyPages.with(|f|{
                let mut current = f[i].ptr;
                let mut past = f[i].ptr;
                while current != 0 as *mut MyPage {
                    // 判断ptr是否在这个page区间之内
                    let temp_ptr1: *const u8 = (current as *const u8).add(mem::size_of::<MyPage>());
                    
                    // 找到对应的页 这个current就是需要的 
                    if ptr.offset_from(temp_ptr1) == 0{
                        // 找到对应页之后进行释放，将链表中间元素去掉

                        if past == current {// 如果是第一个，那么就相当于链表头换成下一个
                            (*(&f[i] as *const PageListHead as *mut PageListHead)).ptr = (*current).next as *mut MyPage;
                        }
                        else {
                            (*past).next = (*current).next as *mut MyPage;
                        }
                        // breakpoint((*(current as *mut MyPage)).size_class as usize);
                        break;
                    }
                    past = current;
                    current = (*current).next as *mut MyPage;
                }
                // 找到后将 current释放
                self.default_allocator.dealloc(current as *mut u8,Layout::from_size_align(layout.size() + mem::size_of::<MyPage>(), 1).unwrap());
            });
            
                        self.update_dealloc(layout.size());
            self.record_msg(layout.size(), 2);
            return ;
        }

        MyPages.with(|f|{
            // 先找到对应的page current
            let mut current = f[i].ptr;
            let mut offset: usize = 0;
            while current != 0 as *mut MyPage {
                // 判断ptr是否在这个page区间之内
                let temp_ptr1: *const u8 = (current as *const u8).add(mem::size_of::<MyPage>());
                let temp_ptr2: *const u8 = (current as *const u8).add(mem::size_of::<MyPage>() + 4096);
                // 找到对应的页 这个current就是需要的 
                if ptr.offset_from(temp_ptr1) >= 0 && ptr.offset_from(temp_ptr2) <= 0 {
                    offset = ptr.offset_from(temp_ptr1) as usize;
                    // breakpoint((*(current as *mut MyPage)).size_class as usize);
                    break;
                }

                current = (*current).next as *mut MyPage;
            }
            
            // 获取位置以及应当减去的值 也就是应当取出的页项

            let (pos, t) = decal_position( offset / f[i].size_class, f[i].size_class);
            
            (*(current as *mut MyPage)).free_list[pos] = (*(current as *mut MyPage)).free_list[pos] - t;
            
            // count.with(|g|{
            //     g.set_temp((*(f[i].ptr as *mut MyPage)).free_list[0] as u64);
            // });

        });
        
        self.update_dealloc(layout.size());
        self.record_msg(layout.size(), 2);
        // self.default_allocator.dealloc(ptr,layout);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if self.get_flag() == false{
            return self.default_allocator.alloc_zeroed(layout);
        }

        self.update_alloc(layout.size());
        self.record_msg(layout.size(), 1);
        return self.default_allocator.alloc_zeroed(layout);
    }
    // 传参传入 旧ptr 旧layout 新大小
    // 输出新ptr
    // 目前是：如果新大小超过了原有的size_class 那么就先deallocate掉 再alloc
    // 如果新大小没超过，就直接返回原来的ptr，保持位置不变
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize
    ) -> *mut u8 {
        if self.get_flag() == false{
            return self.default_allocator.realloc(ptr, layout, new_size);
        }
        
        // 如果旧layout大小与newsize在同一size_class级别之内，就不变，直接返回旧ptr
        let i = find_block(layout.size());
        let j = find_block(new_size);
        if i == j { return ptr; }
        // 如果两者不在同一级，就dealloc掉然后再新alloc
        self.dealloc(ptr, layout);
        let new_layout = Layout::from_size_align(new_size, 1).unwrap();
        return self.alloc(new_layout);


        self.update_alloc(new_size);
        self.record_msg(new_size, 1);
        self.update_dealloc(layout.size());
        self.record_msg(layout.size(), 2);
        // return self.default_allocator.realloc(ptr, layout, new_size);
    }
}
// 测试宏
#[macro_export]
macro_rules! print_cost{
    ($function: ident($($arg:tt)*))=>{
        {
            ALLOCATOR.init();
            let result = $function($($arg)*);
                //ALLOCATOR.set_flag(false);
            ALLOCATOR.finish();
            result
        }
    }
}

