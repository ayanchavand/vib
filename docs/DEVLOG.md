# Devlog

## Day 1 — 2025-12-22

Hi,

So this was the longest sprint I have ever done on a project. I worked almost 11 hours straight.

The idea came from how freakin annoying it is to set up SMB shares on Windows and wondering if there was a better way. The second frustration was using File Explorer and how insanely laggy and unoptimized it is in general, but it is crazy how bad it works for SMB in particular. That was the origin of this project.

I have never worked on any serious project with Rust before, so getting used to all the borrowing rules was a bit painful. I come from OOP languages, so using Rust in a more semi-functional paradigm is something I really liked. By the end of it, I actually started to love how the code structure Rust forces you into writing is more hygienic and disciplined.

I realized that before I could make SMB shares work, I first needed to make a working file explorer alternative with a TUI. That is where most of my time went, implementing basic file browsing features and getting the core interaction right.

Overall, I am quite happy with the progress I have made so far. Let’s see where we go tomorrow.


## Day 2 — 2025-12-23

Hi,

Today was less about writing code and more about thinking through the direction of the project. The plans have changed a bit.

I initially wanted to add SMB support once the TUI file browser was in a decent state. But after spending time looking into Rust SMB client bindings, it became clear that this was going to be far more complicated than I expected. The libraries are either poorly documented, overly complex, or both. I’m sure it’s possible to make it work, but it felt like I would be fighting the tooling instead of actually building the thing I wanted.

That’s when I started questioning whether SMB was even the right approach in the first place.
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
All new ideas must be recorded under *Future Work* and not added to the current scope.

## Day 2 — 2025-12-23

Hi,

Today was less about writing code and more about thinking through the direction of the project. The plans have changed a bit.

I initially wanted to add SMB support once the TUI file browser was in a decent state. But after spending time looking into Rust SMB client bindings, it became clear that this was going to be far more complicated than I expected. The libraries are either poorly documented, overly complex, or both. I’m sure it’s possible to make it work, but it felt like I would be fighting the tooling instead of actually building the thing I wanted.

That’s when I started questioning whether SMB was even the right approach in the first place.

What I actually want is a fast way to browse files on machines I already have access to, without mounting network shares or SSH-ing again and again just to check what’s where. SMB is one solution to that problem, but it’s also archaic, heavy, and annoying to set up, especially on Windows.

So the plan shifted.

Instead of relying on SMB, the new idea is to run a lightweight, dockerized file server written in Go on the host machine. This server would expose selected directories and handle all filesystem interaction. The Rust TUI would simply connect to it over the network, receive structured data, and focus purely on rendering and interaction.

This approach has a few clear advantages. I get full control over the protocol and data model, there’s no OS-level mounting involved, and setup can realistically be as simple as a Docker Compose file. I also don’t have to deal with poorly documented SMB bindings in Rust anymore.

I don’t know yet how this will compare to SMB in terms of raw file transfer performance, but honestly, SMB already performs badly enough in real-world usage that I doubt this can be meaningfully worse. Even if it is slightly slower on paper, the simplicity and predictability feel like a much better tradeoff.

More importantly, this feels like a system I can actually reason about end-to-end. The complexity is intentional and explicit, not inherited from legacy protocols.

Tomorrow, the focus will be on properly defining the server–client boundary before diving back into implementation.

What I actually want is a fast way to browse files on machines I already have access to, without mounting network shares or SSH-ing again and again just to check what’s where. SMB is one solution to that problem, but it’s also archaic, heavy, and annoying to set up—especially on Windows.

So the plan shifted.

Instead of relying on SMB, the new idea is to run a lightweight, dockerized file server written in Go on the host machine. This server would expose selected directories and handle all filesystem interaction. The Rust TUI would simply connect to it over the network, receive structured data, and focus purely on rendering and interaction.

This approach has a few clear advantages. I get full control over the protocol and data model, there’s no OS-level mounting involved, and setup can realistically be as simple as a Docker Compose file. I also don’t have to deal with poorly documented SMB bindings in Rust anymore.

I don’t know yet how this will compare to SMB in terms of raw file transfer performance, but honestly, SMB already performs badly enough in real-world usage that I doubt this can be meaningfully worse. Even if it is slightly slower on paper, the simplicity and predictability feel like a much better tradeoff.

More importantly, this feels like a system I can actually reason about end-to-end. The complexity is intentional and explicit, not inherited from legacy protocols.

Tomorrow, the focus will be on properly defining the server–client boundary before diving back into implementation.
