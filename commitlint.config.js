export default {
    extends: ['@commitlint/config-conventional'],
    ignores: [
        (message) => /^build\(deps(?:-dev)?\):/.test(message)
    ]
};
