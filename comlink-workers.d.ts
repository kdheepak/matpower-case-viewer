declare module 'comlink:@workers/matpower' {
  const mod: () => import('comlink').Remote<typeof import('./src/workers/matpower')>
  export default mod
}
