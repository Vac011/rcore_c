use bit_field::BitField;
use crate::config::PRIO_NUM;

/// 协程优先级位图
#[derive(Clone, Copy)]
pub struct  BitMap(pub u8);

impl BitMap {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn update(&mut self, prio: usize, val: bool) {
        self.0.set_bit(prio-1, val);
    }


    pub fn get(&mut self, id: usize) -> bool {
        self.0.get_bit(id-1)
    }
    /// 获取最高优先级
    pub fn get_priority(&self) -> usize {
        for i in 0..PRIO_NUM  {
            if self.0.get_bit(i) {
                return i+1;
            }
        }
        PRIO_NUM
    }
    /// 
    pub fn get_val(&self) -> u8 {
        self.0
    }
}

