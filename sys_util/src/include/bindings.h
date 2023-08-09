#include <sys/syscall.h>
#include <linux/fs.h>
#include <linux/btrfs.h>
#include <linux/capability.h>

// issue in rust bindgen not yet solved: functional C macros are not expanded: See https://github.com/rust-lang/rust-bindgen/issues/753

const __u64 _BTRFS_IOC_SNAP_CREATE = BTRFS_IOC_SNAP_CREATE;

const __u64 _BTRFS_IOC_DEFRAG = BTRFS_IOC_DEFRAG;

const __u64 _BTRFS_IOC_RESIZE = BTRFS_IOC_RESIZE;

const __u64 _BTRFS_IOC_SCAN_DEV = BTRFS_IOC_SCAN_DEV;

/* trans start and trans end are dangerous, and only for
 * use by applications that know how to avoid the
 * resulting deadlocks
 */
const __u64 _BTRFS_IOC_TRANS_START = BTRFS_IOC_TRANS_START;
const __u64 _BTRFS_IOC_TRANS_END = BTRFS_IOC_TRANS_END;
const __u64 _BTRFS_IOC_SYNC = BTRFS_IOC_SYNC;

const __u64 _BTRFS_IOC_CLONE = BTRFS_IOC_CLONE;
const __u64 _BTRFS_IOC_ADD_DEV = BTRFS_IOC_ADD_DEV;

const __u64 _BTRFS_IOC_RM_DEV = BTRFS_IOC_RM_DEV;

const __u64 _BTRFS_IOC_BALANCE = BTRFS_IOC_BALANCE;

const __u64 _BTRFS_IOC_CLONE_RANGE = BTRFS_IOC_CLONE_RANGE;

const __u64 _BTRFS_IOC_SUBVOL_CREATE = BTRFS_IOC_SUBVOL_CREATE;

const __u64 _BTRFS_IOC_SNAP_DESTROY = BTRFS_IOC_SNAP_DESTROY;

const __u64 _BTRFS_IOC_DEFRAG_RANGE = BTRFS_IOC_DEFRAG_RANGE;

const __u64 _BTRFS_IOC_TREE_SEARCH = BTRFS_IOC_TREE_SEARCH;

const __u64 _BTRFS_IOC_TREE_SEARCH_V2 = BTRFS_IOC_TREE_SEARCH_V2;

const __u64 _BTRFS_IOC_INO_LOOKUP = BTRFS_IOC_INO_LOOKUP;

const __u64 _BTRFS_IOC_DEFAULT_SUBVOL = BTRFS_IOC_DEFAULT_SUBVOL;

const __u64 _BTRFS_IOC_SPACE_INFO = BTRFS_IOC_SPACE_INFO;

const __u64 _BTRFS_IOC_START_SYNC = BTRFS_IOC_START_SYNC;

const __u64 _BTRFS_IOC_WAIT_SYNC = BTRFS_IOC_WAIT_SYNC;

const __u64 _BTRFS_IOC_SNAP_CREATE_V2 = BTRFS_IOC_SNAP_CREATE_V2;

const __u64 _BTRFS_IOC_SUBVOL_CREATE_V2 = BTRFS_IOC_SUBVOL_CREATE_V2;

const __u64 _BTRFS_IOC_SUBVOL_GETFLAGS = BTRFS_IOC_SUBVOL_GETFLAGS;

const __u64 _BTRFS_IOC_SUBVOL_SETFLAGS = BTRFS_IOC_SUBVOL_SETFLAGS;

const __u64 _BTRFS_IOC_SCRUB = BTRFS_IOC_SCRUB;

const __u64 _BTRFS_IOC_SCRUB_CANCEL = BTRFS_IOC_SCRUB_CANCEL;

const __u64 _BTRFS_IOC_SCRUB_PROGRESS = BTRFS_IOC_SCRUB_PROGRESS;

const __u64 _BTRFS_IOC_DEV_INFO = BTRFS_IOC_DEV_INFO;

const __u64 _BTRFS_IOC_FS_INFO = BTRFS_IOC_FS_INFO;

const __u64 _BTRFS_IOC_BALANCE_V2 = BTRFS_IOC_BALANCE_V2;

const __u64 _BTRFS_IOC_BALANCE_CTL = BTRFS_IOC_BALANCE_CTL;

const __u64 _BTRFS_IOC_BALANCE_PROGRESS = BTRFS_IOC_BALANCE_PROGRESS;

const __u64 _BTRFS_IOC_INO_PATHS = BTRFS_IOC_INO_PATHS;

const __u64 _BTRFS_IOC_LOGICAL_INO = BTRFS_IOC_LOGICAL_INO;

const __u64 _BTRFS_IOC_SET_RECEIVED_SUBVOL = BTRFS_IOC_SET_RECEIVED_SUBVOL;

const __u64 _BTRFS_IOC_SEND = BTRFS_IOC_SEND;

const __u64 _BTRFS_IOC_DEVICES_READY = BTRFS_IOC_DEVICES_READY;

const __u64 _BTRFS_IOC_QUOTA_CTL = BTRFS_IOC_QUOTA_CTL;

const __u64 _BTRFS_IOC_QGROUP_ASSIGN = BTRFS_IOC_QGROUP_ASSIGN;

const __u64 _BTRFS_IOC_QGROUP_CREATE = BTRFS_IOC_QGROUP_CREATE;

const __u64 _BTRFS_IOC_QGROUP_LIMIT = BTRFS_IOC_QGROUP_LIMIT;

const __u64 _BTRFS_IOC_QUOTA_RESCAN = BTRFS_IOC_QUOTA_RESCAN;

const __u64 _BTRFS_IOC_QUOTA_RESCAN_STATUS = BTRFS_IOC_QUOTA_RESCAN_STATUS;

const __u64 _BTRFS_IOC_QUOTA_RESCAN_WAIT = BTRFS_IOC_QUOTA_RESCAN_WAIT;

const __u64 _BTRFS_IOC_GET_FSLABEL = BTRFS_IOC_GET_FSLABEL;

const __u64 _BTRFS_IOC_SET_FSLABEL = BTRFS_IOC_SET_FSLABEL;

const __u64 _BTRFS_IOC_GET_DEV_STATS = BTRFS_IOC_GET_DEV_STATS;

const __u64 _BTRFS_IOC_DEV_REPLACE = BTRFS_IOC_DEV_REPLACE;

const __u64 _BTRFS_IOC_FILE_EXTENT_SAME = BTRFS_IOC_FILE_EXTENT_SAME;

const __u64 _BTRFS_IOC_GET_FEATURES = BTRFS_IOC_GET_FEATURES;

const __u64 _BTRFS_IOC_SET_FEATURES = BTRFS_IOC_SET_FEATURES;

const __u64 _BTRFS_IOC_GET_SUPPORTED_FEATURES = BTRFS_IOC_GET_SUPPORTED_FEATURES;

const __u64 _BTRFS_IOC_RM_DEV_V2 = BTRFS_IOC_RM_DEV_V2;

const __u64 _BTRFS_IOC_LOGICAL_INO_V2 = BTRFS_IOC_LOGICAL_INO_V2;