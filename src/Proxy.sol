pragma solidity 0.8.20;

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
        address _setter,
        address _admin,
        address _vault
    ) {
        //0xde523c1d72dcdef5a38555718692f2b21869e2f35bbcf0debfef01259e2deef7
        setSlotVal(0, _solver);
        //0x90685e9ea439b19f41929775b048269211eec02ce328e0f38490780ccd1483af
        setSlotVal(1, _setter);
        //0xa02c041eff3d49b6a99c1e129c48fb4d9adffac94b19c58bcf81c2facce2f75e
        setSlotVal(2, _admin);
        //0x782dba7347fd60e063937f66efec585ac87485d71d6d36306c305aad7fa1c3c3
        setSlotVal(3, _vault);
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
