function main() {
    const a = 1
    const b = 2
    throw new Error('ee')
    return a + b
}

try {
    main()
} catch (error) {
    throw new TypeError(error.message)
}
