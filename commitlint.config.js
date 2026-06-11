export default {
    extends: ['@commitlint/config-conventional'],
    ignores: [
        (message) => /^(build|chore)\(deps(?:-dev)?\):/.test(message)
    ]
};
