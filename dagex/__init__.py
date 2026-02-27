from .dagex import *  # type: ignore  (compiled Rust extension)
from .analysis import JointDistribution, joint_from_stat as joint

__doc__ = dagex.__doc__  # type: ignore
if hasattr(dagex, "__all__"):  # type: ignore
    __all__ = dagex.__all__  # type: ignore
