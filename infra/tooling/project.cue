package project

// This is development infrastructure, not a Rust implementation scaffold.
// Shaka discovers project declarations at <slot>/<name>/project.cue.
#Project & {
	name: "cuestacean"
	kind: "infra"
	ci: validate: {}
}
