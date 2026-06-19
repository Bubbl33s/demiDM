name: Feature Request
description: Suggest a new feature or enhancement
title: "[Feature]: "
labels: ["enhancement", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for suggesting a feature! Please provide as much detail as possible.

  - type: textarea
    id: problem
    attributes:
      label: Problem Statement
      description: What problem does this feature solve? What's the motivation?
      placeholder: I'm always frustrated when...
    validations:
      required: true

  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
      description: Describe the solution you'd like to see
      placeholder: A clear description of what you want to happen
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
      description: Any alternative solutions or features you've considered
    validations:
      required: false

  - type: dropdown
    id: scope
    attributes:
      label: Component
      description: Which part of DemiDM would this affect?
      options:
        - Authentication (PAM)
        - UI/Rendering
        - Lua API
        - Widgets
        - Graphics/Backgrounds
        - Configuration
        - Session Management
        - Other
    validations:
      required: true

  - type: textarea
    id: context
    attributes:
      label: Additional Context
      description: Any other context, mockups, or references
    validations:
      required: false

  - type: checkboxes
    id: willing
    attributes:
      label: Contribution
      description: Are you willing to help implement this feature?
      options:
        - label: I would be willing to submit a PR for this feature
        - label: I can help with testing
        - label: I'm just suggesting, won't contribute

  - type: checkboxes
    id: terms
    attributes:
      label: Code of Conduct
      description: By submitting this request, you agree to follow our Code of Conduct
      options:
        - label: I agree to follow this project's Code of Conduct
          required: true
