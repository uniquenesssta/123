# Application Screen Map

## Project

Repository: uniquenesssta/123

Current UI audit baseline: v0.23.0

## Application Structure

The application is a Tauri desktop platform with Rust workspace modules and frontend entry points. Existing modules include model API, model stub, application, domain, analysis, review, research and persistence layers.

## Primary Product Areas

### 1. Dashboard / Workspace

Purpose:
- Global operational overview
- Navigation entry
- Status summary

Required UI states:
- Empty workspace
- Loading data
- Permission restricted
- Data loaded
- Error recovery

### 2. Match Analysis

Purpose:
- Match data exploration
- Analysis workflow
- Review process

Components required:
- Match card
- Filter panel
- Analysis timeline
- Result panel
- Evidence display

### 3. Team Management

Purpose:
- Team entities
- Team resources
- Player relationships

Components required:
- Search
- Table
- Detail drawer
- Edit form
- Import workflow

### 4. Player Management

Purpose:
- Player entities
- Role and relationship management

Components required:
- Player table
- Profile panel
- Relationship graph
- History records

### 5. API / Model Workspace

Purpose:
- External model connection
- Runtime diagnostics
- Provider status

Components required:
- Provider selector
- Request panel
- Response viewer
- Logs
- Error diagnostics

### 6. History / Review

Purpose:
- Historical analysis
- Review packages
- Settlement records

Components required:
- Timeline
- Snapshot viewer
- Comparison view

## Global Interaction Requirements

Every interactive element must define:

- Default state
- Hover state
- Active state
- Disabled state
- Loading state
- Error response
- Success response

## Next Audit Step

Create detailed User Flow Map and Component Dependency Map before Figma implementation.
