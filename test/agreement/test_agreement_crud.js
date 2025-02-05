const {
  buildConfig,
  buildRunner,
  buildPlayer,
} = require('../init')

const runner = buildRunner()

const config = buildConfig()

const exampleEntry = {
  name: 'test agreement',
  created: new Date(),
  note: 'just testing, nothing was rly agreed',
}
const updatedExampleEntry = {
  name: 'updated agreement',
  created: new Date(Date.now() + 3600000),
  note: 'updated the agreement to something else',
}

runner.registerScenario('Agreement record API', async (s, t) => {
  const alice = await buildPlayer(s, config, ['agreement'])

  let createResp = await alice.graphQL(`
    mutation($rs: AgreementCreateParams!) {
      res: createAgreement(agreement: $rs) {
        agreement {
          id
          revisionId
        }
      }
    }
  `, {
    rs: exampleEntry,
  })
  await s.consistency()
  t.ok(createResp.data.res.agreement.id, 'record created')
  const aId = createResp.data.res.agreement.id
  const r1Id = createResp.data.res.agreement.revisionId

  let getResp = await alice.graphQL(`
    query($id: ID!) {
      res: agreement(id: $id) {
        id
        revisionId
        name
        created
        note
      }
    }
  `, {
    id: aId,
  })
  t.deepEqual(getResp.data.res, { 'id': aId, revisionId: r1Id, ...exampleEntry }, 'record read OK')

  const updateResp = await alice.graphQL(`
    mutation($rs: AgreementUpdateParams!) {
      res: updateAgreement(agreement: $rs) {
        agreement {
          id
          revisionId
        }
      }
    }
  `, {
    rs: { revisionId: r1Id, ...updatedExampleEntry },
  })
  await s.consistency()
  t.equal(updateResp.data.res.agreement.id, aId, 'record updated')
  const r2Id = updateResp.data.res.agreement.revisionId

  // now we fetch the Entry again to check that the update was successful
  const updatedGetResp = await alice.graphQL(`
    query($id: ID!) {
      res: agreement(id: $id) {
        id
        revisionId
        created
        name
        note
      }
    }
  `, {
    id: aId,
  })
  t.deepEqual(updatedGetResp.data.res, { id: aId, revisionId: r2Id, created: exampleEntry.created, ...updatedExampleEntry }, 'record updated OK')

  const deleteResult = await alice.graphQL(`
    mutation($id: ID!) {
      res: deleteAgreement(revisionId: $id)
    }
  `, {
    id: r2Id,
  })
  await s.consistency()
  t.equal(deleteResult.data.res, true)

  const queryForDeleted = await alice.graphQL(`
    query($id: ID!) {
      res: agreement(id: $id) {
        id
      }
    }
  `, {
    id: aId,
  })
  t.equal(queryForDeleted.errors.length, 1, 'querying deleted record is an error')
  t.notEqual(-1, queryForDeleted.errors[0].message.indexOf('No entry at this address'), 'correct error reported')
})

runner.run()
