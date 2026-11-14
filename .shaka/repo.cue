package repopolicy

#RepoPolicy & {
	defaultBranch: "main"
	merge: {
		rebase:              true
		merge:               false
		squash:              false
		deleteBranchOnMerge: true
		autoMerge:           false
	}
	issues: true
	branchProtection: {
		requiredChecks: ["cuestacean validate"]
		strictStatusChecks: false
		allowForcePush:     false
		allowDeletion:      false
	}
	security: dependabotAlerts: true
}
