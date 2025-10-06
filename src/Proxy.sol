// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

//0xffdea0388ca4a843ade7b3e2c2bc13081b779a3f6d573e3cdda653f4db50c868
bytes32 constant SLOT_VAULT = bytes32(uint256(keccak256(abi.encodePacked("passport.superposition.impl.vault"))) - 1);

//0xb1071da564e73d02ae6815965358af289c23a15d2fe16d58dc4a81c3101b4d79
bytes32 constant SLOT_APPLY = bytes32(uint256(keccak256(abi.encodePacked("passport.superposition.impl.apply"))) - 1);

contract Proxy {
    function getSlot(uint8 x) internal pure returns (bytes32) {
        return bytes32(uint256(keccak256(abi.encodePacked("passport.superposition.impl.slot.", x))) - 1);
    }

    function getSlotVal(uint8 x) internal view returns (address addr) {
        bytes32 s = getSlot(x);
        assembly {
            addr := sload(s)
        }
    }

    function setSlotVal(uint8 x, address addr) internal {
        bytes32 s = getSlot(x);
        assembly {
            sstore(s, addr)
        }
    }

    constructor(
        address _solver,
        address _apply,
        address _setter,
        address _admin,
        address _vault
    ) {
        bytes32 slotVault = SLOT_VAULT;
        bytes32 slotApply = SLOT_APPLY;
        //0xde523c1d72dcdef5a38555718692f2b21869e2f35bbcf0debfef01259e2deef7
        setSlotVal(0, _solver);
        //0x90685e9ea439b19f41929775b048269211eec02ce328e0f38490780ccd1483af
        setSlotVal(1, _setter);
        //0xa02c041eff3d49b6a99c1e129c48fb4d9adffac94b19c58bcf81c2facce2f75e
        setSlotVal(2, _admin);
        assembly {
            sstore(slotVault, _vault)
            sstore(slotApply, _apply)
        }
    }

    fallback() external {
        address to = getSlotVal(uint8(msg.data[0]));
        assembly {
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), to, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch result
            case 0 {
                revert(0, returndatasize())
            }
            default {
                return(0, returndatasize())
            }
        }
    }
}
