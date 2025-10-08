// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "../src/Proxy.sol";

interface IAF {
    function deployStylusCode(string calldata artifact) external returns (address);
}

contract TestPassportE2E is Test {
    Proxy proxy;

    address solver;
    address applyy;
    address setter;
    address admin;
    address vault;

    function setUp() public {
        admin = IAF(address(vm)).deployStylusCode("admin.passport-superposition-so.wasm");
        applyy = IAF(address(vm)).deployStylusCode("apply.passport-superposition-so.wasm");
        setter = IAF(address(vm)).deployStylusCode("setter.passport-superposition-so.wasm");
        solver = IAF(address(vm)).deployStylusCode("solver.passport-superposition-so.wasm");
        vault = IAF(address(vm)).deployStylusCode("vault.passport-superposition-so.wasm");
        proxy = new Proxy(solver, applyy, setter, admin, vault);
    }

    function testDeploymentsWereOkay() external view {
        assertNotEq(address(0), admin);
        assertNotEq(address(0), applyy);
        assertNotEq(address(0), setter);
        assertNotEq(address(0), solver);
        assertNotEq(address(0), vault);
        assertNotEq(address(0), address(proxy));
    }
}
