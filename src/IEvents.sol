// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

interface IEvents {
    event KeyRegistered(
        uint64 indexed offset,
        bytes32 indexed key,
        address indexed owner
    );
}
