use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[macro_export]
macro_rules! define_mirror_struct {
    (
        // 1. 公共属性 (应用到两个 Struct)
        $(#[$common_meta:meta])*

        // 2. 左侧 Struct (通常带有特殊属性，如 untagged)
        $(#[$left_meta:meta])* // 左侧独有属性
        $vis:vis struct $LeftName:ident

        // 3. 映射符号
        =>

        // 4. 右侧 Struct (通常是干净的镜像)
        $(#[$right_meta:meta])* // 右侧独有属性 (可选)
        $RightName:ident

        // 5. 字段定义 (必须显式解析字段以生成转换代码)
        {
            $($field_vis:vis $field_name:ident : $field_type:ty),* $(,)?
        }
    ) => {
        // === 生成左侧 Struct ===
        $(#[$common_meta])*
        $(#[$left_meta])*
        $vis struct $LeftName {
            $($field_vis $field_name : $field_type),*
        }

        // === 生成右侧 Struct ===
        $(#[$common_meta])*
        $(#[$right_meta])*
        $vis struct $RightName {
            $($field_vis $field_name : $field_type),*
        }

        // === 自动生成 Left -> Right 的转换 ===
        impl From<$LeftName> for $RightName {
            fn from(source: $LeftName) -> Self {
                Self {
                    $($field_name: source.$field_name),*
                }
            }
        }

        // === 自动生成 Right -> Left 的转换 ===
        impl From<$RightName> for $LeftName {
            fn from(source: $RightName) -> Self {
                Self {
                    $($field_name: source.$field_name),*
                }
            }
        }
    };
}

// ==========================================
// 使用示例
// ==========================================
