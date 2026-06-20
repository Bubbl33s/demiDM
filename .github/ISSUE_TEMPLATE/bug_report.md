name: Bug Report
description: Report a bug or unexpected behavior
title: "[Bug]: "
labels: ["bug", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to report a bug! Please fill out the sections below.

  - type: textarea
    id: description
    attributes:
      label: Bug Description
      description: A clear and concise description of the bug
      placeholder: What happened?
    validations:
      required: true

  - type: textarea
    id: reproduction
    attributes:
      label: Steps to Reproduce
      description: Steps to reproduce the behavior
      placeholder: |
        1. Configure demidm with '...'
        2. Start the display manager
        3. Try to login with '...'
        4. See error
    validations:
      required: true

  - type: textarea
    id: expected
    attributes:
      label: Expected Behavior
      description: What did you expect to happen?
    validations:
      required: true

  - type: textarea
    id: actual
    attributes:
      label: Actual Behavior
      description: What actually happened?
    validations:
      required: true

  - type: textarea
    id: logs
    attributes:
      label: Logs
      description: Relevant log output from /tmp/demidm.log
      render: shell
    validations:
      required: false

  - type: textarea
    id: environment
    attributes:
      label: Environment
      description: System information
      value: |
        - OS: [e.g., Arch Linux, Ubuntu 22.04]
        - Kernel: [e.g., 6.1.0]
        - Rust version: [e.g., 1.88.0]
        - DemiDM version: [e.g., 0.1.0 or commit hash]
    validations:
      required: true

  - type: textarea
    id: additional
    attributes:
      label: Additional Context
      description: Any other context, screenshots, or configuration that might be relevant
    validations:
      required: false

  - type: checkboxes
    id: terms
    attributes:
      label: Code of Conduct
      description: By submitting this issue, you agree to follow our Code of Conduct
      options:
        - label: I agree to follow this project's Code of Conduct
          required: true
