# RFC Creation Checklist

Use this checklist when creating a new RFC.

## Pre-Writing Checklist

- [ ] **Identify the problem** - What specific issue does this address?
- [ ] **Research alternatives** - What existing solutions exist?
- [ ] **Scope definition** - Is this one RFC or should it be split?
- [ ] **Target audience** - Who needs to review this?
- [ ] **Timeline** - What's the target version/release?

## RFC Document Checklist

### Header Section
- [ ] RFC number assigned (sequential)
- [ ] Descriptive title
- [ ] Status set to "Draft"
- [ ] Author(s) listed
- [ ] Creation date
- [ ] Target version

### Summary Section
- [ ] One paragraph overview
- [ ] Clear value proposition
- [ ] Main benefits listed

### Motivation Section
- [ ] Current state analysis
- [ ] Problems/limitations identified
- [ ] Industry comparison (if relevant)
- [ ] User needs documented

### Design Section
- [ ] Complete configuration/API example
- [ ] All new fields documented
- [ ] Type information for each field
- [ ] Default values specified
- [ ] Error handling described
- [ ] Edge cases considered

### Backward Compatibility
- [ ] Breaking changes identified
- [ ] Migration path documented
- [ ] Migration commands provided
- [ ] Before/after examples

### Implementation Plan
- [ ] Phased approach
- [ ] Tasks broken down
- [ ] Dependencies identified
- [ ] Target versions for each phase

### References
- [ ] Related documentation linked
- [ ] External resources cited

### Changelog
- [ ] Initial entry added

## Review Checklist

Before moving to "Review" status:

- [ ] All sections complete
- [ ] Examples are realistic and working
- [ ] No TODOs or placeholders
- [ ] Spelling and grammar checked
- [ ] Formatting consistent
- [ ] Links verified

## Post-Acceptance Checklist

After RFC is accepted:

- [ ] Create implementation tracker (if complex)
- [ ] Create feature branch
- [ ] Update project roadmap
- [ ] Assign tasks to team members
- [ ] Set up milestone tracking

## Implementation Completion Checklist

Before marking as "Implemented":

- [ ] All phases completed
- [ ] Tests passing
- [ ] Documentation updated
- [ ] Migration guide available
- [ ] Release notes prepared
- [ ] RFC status updated
