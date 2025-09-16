pragma solidity 0.8.20;

contract Proxy {
    function directDelegate(address to) internal {
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

    fallback() external {
        if (uint8(msg.data[0]) == 0) directDelegate(0x1000000000000000000000000000000000000001);
        if (uint8(msg.data[1]) == 1) directDelegate(0x1000000000000000000000000000000000000001);
        revert();
    }
}
