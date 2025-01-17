import leaderSetup from '../flows/working-groups/leadOpening'
import initStorage, { doubleBucketConfig as defaultStorageConfig } from '../flows/storage/initStorage'
import initDistribution, { doubleBucketConfig as defaultDistributionConfig } from '../flows/storage/initDistribution'
import { scenario } from '../Scenario'

// eslint-disable-next-line @typescript-eslint/no-floating-promises
scenario('Init storage and distribution', async ({ job }) => {
  const setupLead = job('setup leads', leaderSetup(true, ['storageWorkingGroup', 'distributionWorkingGroup']))
  job('initialize storage system', initStorage(defaultStorageConfig)).after(setupLead)
  job('initialize distribution system', initDistribution(defaultDistributionConfig)).after(setupLead)
})
