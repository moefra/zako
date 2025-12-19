use hone::{HoneResult, status::NodeData};

use crate::{
    computer::ZakoComputeContext, context::BuildContext, file_artifact::FileArtifact,
    intern::InternedAbsolutePath, node_value::ZakoValue, path::interned::InternedNeutralPath,
    pattern::InternedPattern,
};

pub async fn compute_file<'c>(
    ctx: &'c ZakoComputeContext<'c>,
    path: &InternedNeutralPath,
) -> HoneResult<NodeData<BuildContext, ZakoValue>> {
    // 1. 路径转换 (Logical -> Physical)
    // 只有在这个函数内部，我们才关心绝对路径
    // 2. 执行 IO (读取磁盘)
    // 可以在这里加 fs watch 的订阅逻辑
    // 检查权限 (Unix only)
    // 3. 存入 CAS (Ingestion)
    // 这一步计算了 Hash，并把数据放入了 BlobStore
    // 4. 构造返回值
    // 这里的 value 包含了 content hash。
    // 如果文件内容变了 -> handle.local_hash 变了 -> value hash 变了 -> 下游触发重算。
    // 5. 计算 Input Hash
    // 对于源文件，Input Hash 就是路径本身的 Hash
    todo!()
}
