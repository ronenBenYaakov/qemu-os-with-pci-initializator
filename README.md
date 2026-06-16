# 🖥️ QEMU PCI Initializer System

## 📌 Overview

This project is a **low-level PCI initialization and device discovery layer running inside QEMU**.

It is designed to simulate and explore how operating systems or bare-metal environments:

* 🧠 Detect PCI devices
* 🔍 Enumerate hardware buses
* ⚙️ Initialize device configurations
* 🧩 Build a structured internal device map

---

## 🎯 Core Purpose

The system acts as a **PCI initialization engine** that runs in a virtualized QEMU environment and:

* Scans PCI configuration space
* Detects devices (GPU, NIC, storage controllers, etc.)
* Builds an internal device registry
* Prepares devices for driver binding

---

## 🏗️ System Architecture

```text
🖥️ QEMU Virtual Machine
        ↓
⚙️ Boot / Kernel Entry
        ↓
🧠 PCI Initializer
        ↓
🔍 PCI Bus Scanner
        ↓
📦 Device Registry
        ↓
🔌 Driver Binding Layer (future)
```

---

## 🔍 What This System Does

### 🧭 PCI Discovery

* Scans PCI buses (0–255)
* Reads device/vendor IDs
* Detects multi-function devices

---

### 🧱 Device Construction

Each detected PCI device is stored as:

* Vendor ID 🏷️
* Device ID 🧩
* Class code ⚙️
* Bus / Slot / Function 📍
* BAR (Base Address Registers) 📦

---

### ⚙️ Initialization Flow

* Enable PCI command register
* Configure memory and I/O access
* Read capabilities (MSI, MSI-X)
* Map BAR regions

---

## 🧬 Internal Model

Each PCI device is represented as:

* 🧾 Identity (vendor/device ID)
* 📍 Location (bus/slot/function)
* 🧠 Class (network, storage, display, etc.)
* 📦 Resource mapping (MMIO / IO ports)
* ⚡ Capability flags

---

## 🔌 Example Flow

```text
[PCI INIT]
→ Scanning bus 0...
→ Found device: 8086:100e (Intel NIC)
→ Found device: 1234:1111 (Virtual GPU)

[REGISTER]
→ Adding to device table

[INIT]
→ Mapping BAR0
→ Enabling bus mastering
→ Device ready
```

---

## 📊 Output Example

```text
🧠 PCI INITIALIZATION COMPLETE

📦 Device Found:
- 00:02.0 | VGA Controller | 1234:1111 | QEMU Virtual GPU
- 00:03.0 | Ethernet Ctrl  | 8086:100e | Intel E1000

⚙️ Total Devices: 2
```
---

## 🧠 Key Features

### 🔍 PCI Scanner

* Direct config space reads
* Multi-bus enumeration
* Device classification

### 🧩 Device Registry

* Internal structured storage
* Lookup by bus/slot/function
* Extensible metadata model

### ⚙️ Initialization Layer

* Enables device capabilities
* Prepares devices for drivers
* Simulates OS boot behavior

---

## 🔄 Roadmap

### 🟢 Phase 1

* PCI enumeration
* device listing
* basic registry

### 🟡 Phase 2

* BAR mapping
* capability parsing
* interrupt detection

### 🔵 Phase 3

* driver binding layer
* virtual device simulation

### 🔴 Phase 4

* mini kernel integration
* full OS-style PCI subsystem

---
