/**
 * Resolvers for Satisfaction record relationships
 *
 * @package: HoloREA
 * @since:   2019-08-31
 */

import { DNAIdMappings, addTypename, DEFAULT_VF_MODULES } from '../types'
import { mapZomeFn } from '../connection'

import {
  Satisfaction,
  EventOrCommitment,
  Intent,
} from '@valueflows/vf-graphql'

async function extractRecordsOrFail (query, subfieldId: string): Promise<any> {
  const val = await query
  if (!val || !val.length || !val[0][subfieldId]) {
    throw new Error('Reference not found')
  }
  return val[0][subfieldId]
}

export default (enabledVFModules: string[] = DEFAULT_VF_MODULES, dnaConfig: DNAIdMappings, conductorUri: string) => {
  const hasObservation = -1 !== enabledVFModules.indexOf("observation")

  const readEvents = mapZomeFn(dnaConfig, conductorUri, 'observation', 'economic_event_index', 'query_economic_events')
  const readCommitments = mapZomeFn(dnaConfig, conductorUri, 'planning', 'commitment_index', 'query_commitments')
  const readIntents = mapZomeFn(dnaConfig, conductorUri, 'planning', 'intent_index', 'query_intents')

  return {
    satisfiedBy: async (record: Satisfaction): Promise<EventOrCommitment> => {
      // :NOTE: this presumes a satisfaction will never be erroneously linked to 2 records
      return (
        await Promise.all([
          extractRecordsOrFail(readCommitments({ params: { satisfies: record.id } }), 'commitment')
            .then(addTypename('Commitment'))
            .catch((e) => e),
        ].concat(hasObservation ? [
          extractRecordsOrFail(readEvents({ params: { satisfies: record.id } }), 'economicEvent')
            .then(addTypename('EconomicEvent'))
            .catch((e) => e),
        ] : []))
      )
      .filter(r => !(r instanceof Error))
      .pop()
    },

    satisfies: async (record: Satisfaction): Promise<Intent> => {
      return (await readIntents({ params: { satisfiedBy: record.id } })).pop()['intent']
    },
  }
}
