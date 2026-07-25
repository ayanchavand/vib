# Software Requirements Specification (SRS)

## 1. Problem Statement

Accessing files on remote machines typically requires repeated SSH sessions, manual file transfers, or heavyweight graphical tools. While protocols like SMB exist, they introduce platform-specific issues, unreliable libraries, and poor terminal-first workflows.

Power users who manage multiple machines—servers, homelabs, remote PCs—lack a fast, keyboard-driven way to browse and access remote files without mounting filesystems or constantly reconnecting via SSH.

The goal of this project is to build a fast, predictable, terminal-based tool that allows efficient browsing and access to files hosted on remote machines running a lightweight file server.

---

## 2. User Model & Assumptions

The primary user is the developer of the tool.

The user is technically proficient, comfortable with terminal-based applications, and regularly works with remote Linux or Windows machines. The user prefers keyboard-driven interaction, values responsiveness over visual polish, and is comfortable running Docker containers or lightweight services on machines they control.

The user operates in trusted environments (local networks, personal servers, VPSes) and is willing to configure basic authentication manually.

---

## 3. Goals and Non-Goals

### Goals

* Provide fast, keyboard-driven browsing of remote files
* Avoid filesystem mounts and OS-level integrations
* Support both Windows and Linux clients
* Maintain a responsive TUI under network latency
* Allow simple configuration and deployment of the remote file server
* Minimize client-side processing by structuring data server-side

### Non-Goals

* Full replacement for SSH or SFTP
* Multi-user permission management
* Advanced filesystem features (ACL editing, locking)
* File synchronization or background syncing
* Search, indexing, or thumbnail generation
* Web-based user interface
* Support for arbitrary third-party file servers

---

## 4. Solution Overview

The system consists of two components:

1. **Remote File Server**
   A lightweight file server written in Go, typically deployed via Docker.
   The server exposes selected directories as logical drives over a simple network API and supports basic authentication.

2. **Terminal User Interface (TUI) Client**
   A cross-platform terminal application written in Rust.
   The client connects to a remote server via its IP address, retrieves structured directory data, and renders a keyboard-driven interface for navigation and file access.

The client remains UI-focused and stateless, while filesystem logic and data structuring are handled server-side.

---

## 5. Functional Requirements

1. The system shall allow the user to configure and connect to a remote file server using an IP address and credentials.
2. The server shall expose one or more configured directories as logical root drives.
3. The client shall list directories and files provided by the server.
4. The client shall allow navigation into and out of directories using keyboard input.
5. The client shall allow previewing or opening files using external terminal viewers.
6. The client shall allow downloading files from the remote server to the local system.
7. The system shall provide clear error messages for connection or authentication failures.
8. The client shall allow refreshing the current directory view.

---

## 6. Non-Functional Requirements

* **Performance:** Directory listings and navigation should feel immediate under normal network conditions.
* **Responsiveness:** The TUI must not block during network operations.
* **Portability:** The client must run on both Windows and Linux.
* **Reliability:** Network failures must be handled gracefully without crashing the UI.
* **Usability:** Core actions must be accessible via single-key bindings.
* **Maintainability:** Clear separation between UI logic, networking, and protocol handling.

---

## 7. Constraints

* Development time is limited to approximately 10 days.
* The project is developed by a single developer.
* The TUI client is written in Rust.
* The remote file server is written in Go.
* The server is intended for trusted environments only.
* The interface is terminal-based only (TUI).

---

## 8. Increment Plan

### Version 0.1

* Local TUI scaffolding
* Mock remote directory listings
* Basic navigation

### Version 0.2

* Network connection handling
* Authentication support
* Error handling and UI state management

### Version 0.3 (MVP)

* File preview via external programs
* File download support
* Server configuration documentation
* Usage instructions

---

## 9. Risks and Unknowns

* Network latency affecting perceived responsiveness
* Defining a stable and minimal server API
* Handling partial failures without UI freezes
* Security implications of exposing filesystem access

---

## 10. Future Work (Out of Scope)

* File upload support
* Multi-user access
* Encryption beyond basic transport security
* Command palette or scripting
* Background transfers
* Alternative client interfaces

---

**Note:**
This SRS is frozen for version 1.
All new ideas must be recorded under *Future Work* and not added to t
