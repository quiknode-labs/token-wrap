import assert from 'node:assert/strict';
import test from 'node:test';

import { address } from '@solana/kit';

import {
    findCanonicalPointerPda,
    getCanonicalDeploymentPointerDecoder,
    getCanonicalDeploymentPointerEncoder,
    getSetCanonicalPointerInstructionDataDecoder,
    getSetCanonicalPointerInstructionDataEncoder,
    identifyTokenWrapInstruction,
    TOKEN_WRAP_PROGRAM_ADDRESS,
    TokenWrapInstruction,
} from '../src/generated';

const PWRAP_PROGRAM_ADDRESS = 'pWrapnbzNPTx9aZPAp3gpxAUrs3H4QQ1GHWMPMbDba2';
const TOKEN_2022_PROGRAM_ADDRESS = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';

test('generated client uses the pWrap identity and all deployed discriminators', () => {
    assert.equal(TOKEN_WRAP_PROGRAM_ADDRESS, PWRAP_PROGRAM_ADDRESS);

    for (let discriminator = 0; discriminator <= 6; discriminator += 1) {
        assert.equal(identifyTokenWrapInstruction(Uint8Array.of(discriminator)), discriminator);
    }

    assert.equal(TokenWrapInstruction.SetCanonicalPointer, 6);
    assert.throws(() => identifyTokenWrapInstruction(Uint8Array.of(7)));
});

test('SetCanonicalPointer codecs match the Rust wire layout', () => {
    const programId = address(PWRAP_PROGRAM_ADDRESS);
    const instructionData = getSetCanonicalPointerInstructionDataEncoder().encode({ programId });

    assert.equal(instructionData.length, 33);
    assert.equal(instructionData[0], 6);
    assert.deepEqual(getSetCanonicalPointerInstructionDataDecoder().decode(instructionData), {
        discriminator: 6,
        programId,
    });

    const accountData = getCanonicalDeploymentPointerEncoder().encode({ programId });
    assert.equal(accountData.length, 32);
    assert.deepEqual(getCanonicalDeploymentPointerDecoder().decode(accountData), { programId });
});

test('canonical pointer PDA matches the independent Solana CLI derivation', async () => {
    const [canonicalPointer, bump] = await findCanonicalPointerPda({
        unwrappedMint: address(TOKEN_2022_PROGRAM_ADDRESS),
    });

    assert.equal(canonicalPointer, '3zVro3w2jWeTr5MYAFyqj35e2GRrUFXZyC9eWLJE8BvF');
    assert.equal(bump, 254);
});
