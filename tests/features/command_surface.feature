Feature: Splitters command surface
  The splitters binary declares map, validate, and extract and reports
  their status without touching the repository.

  Scenario: The top-level help lists every command
    When the operator runs "splitters --help"
    Then the exit status is 0
    And the commands section lists "map", "validate", and "extract"
    And standard error is empty

  Scenario: The map command documents --base
    When the operator runs "splitters map --help"
    Then the exit status is 0
    And the options section lists "--base"

  Scenario: The map command documents --manifest-dir
    When the operator runs "splitters map --help"
    Then the exit status is 0
    And the options section lists "--manifest-dir"

  Scenario: The validate command documents the proposal operand
    When the operator runs "splitters validate --help"
    Then the exit status is 0
    And the options section lists "<PROPOSAL>"

  Scenario: The validate command documents --check
    When the operator runs "splitters validate --help"
    Then the exit status is 0
    And the options section lists "--check"

  Scenario: The extract command documents --branch
    When the operator runs "splitters extract --help"
    Then the exit status is 0
    And the options section lists "--branch"

  Scenario: The extract command documents --publish
    When the operator runs "splitters extract --help"
    Then the exit status is 0
    And the options section lists "--publish"

  Scenario: A well-formed command reports that it is not implemented yet
    When the operator runs "splitters validate add-parser --check 'make test'"
    Then the exit status is 2
    And standard output is empty
    And standard error says "splitters validate: not implemented yet"

  Scenario: A missing proposal is a usage error
    When the operator runs "splitters extract"
    Then the exit status is 2
    And standard output is empty
    And standard error names "<PROPOSAL>"

  Scenario: A boolean flag rejects a detached value
    When the operator runs "splitters extract add-parser --publish false"
    Then the exit status is 2
    And standard output is empty
    And standard error is a usage diagnostic

  Scenario: The documented help matches the binary
    Given the users' guide example "splitters-help"
    When the operator runs the documented command
    Then standard output equals the documented output
