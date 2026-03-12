import { spawnSync } from 'node:child_process'
import path from 'node:path'
import { desktopE2eRoot } from '../utils/resolve-paths.mjs'

const forwardedArgs = process.argv.slice(2).filter((argument) => argument !== '--')
const result = spawnSync('wdio', ['run', path.resolve(desktopE2eRoot, 'wdio.conf.mjs'), ...forwardedArgs], {
  cwd: desktopE2eRoot,
  stdio: 'inherit',
  shell: true,
})

process.exit(result.status ?? 1)
