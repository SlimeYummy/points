import assert from 'node:assert';
import { describe, expect, it } from '@jest/globals';
import {
    parseArray,
    parseBool,
    parseBoolArray,
    parseFloat,
    parseFloatArray,
    parseFloatRange,
    parseID,
    parseIDArray,
    parseInt,
    parseIntArray,
    parseIntRange,
    parseString,
} from '../src/common';

// prettier-ignore
describe('common', () => {
    it('parseBool()', () => {
        expect(() => parseBool(null as any, '?'))
            .toThrow('?: must be a boolean');

        assert.equal(parseBool(true, '?'), true);
        assert.equal(parseBool(0, '?'), false);
    });

    it('parseInt', () => {
        expect(() => parseInt(null as any, '?'))
            .toThrow('?: must be a int');

        expect(() => parseInt(null as any, '?', { allow_bool: true }))
            .toThrow('?: must be a int/boolean');

        expect(() => parseInt(0, '?', { min: 1 }))
            .toThrow('?: must >= 1');

        expect(() => parseInt(11, '?', { max: 10 }))
            .toThrow('?: must <= 10');

        assert.equal(parseInt(5, '?'), 5);
        assert.equal(parseInt(true, '?', { allow_bool: true }), 1);
    });

    it('parseFloat', () => {
        expect(() => parseFloat(null as any, '?'))
            .toThrow('?: must be a float');
        
        expect(() => parseFloat(0, '?', { min: 10 }))
            .toThrow('?: must >= 10');
        
        expect(() => parseFloat(11, '?', { max: 9 }))
            .toThrow('?: must <= 9');
        
        assert.equal(parseFloat(5.5, '?'), 5.5);
        assert.equal(parseFloat('5.5%', '?'), 0.055);
    });

    it('parseArray', () => {
        expect(() => parseArray(null as any, '?', () => null))
            .toThrow('?: must be an array');
        
        expect(() => parseArray([], '?', () => null, { len: 1 }))
            .toThrow('?: length must = 1');
        
        expect(() => parseArray([1], '?', () => null, { min_len: 2 }))
            .toThrow('?: length must >= 2');
        
        expect(() => parseArray([1, 2, 3], '?', () => null, { max_len: 2 }))
            .toThrow('?: length must <= 2');

        assert.deepEqual(parseArray([1, 2, 3], '?', (n: number) => n + 1), [2, 3, 4]);
        assert.deepEqual(parseArray([1, 2, 3], '?', (n: number) => n + 1), [2, 3, 4]);
    })

    it('parseBoolArray', () => {
        expect(() => parseBoolArray(null as any, '?'))
            .toThrow('?: must be an array');
        
        expect(() => parseBoolArray([null] as any, '?'))
            .toThrow('?[0]: must be a boolean');
        
        expect(() => parseBoolArray([], '?', { len: 1 }))
            .toThrow('?: length must = 1');
        
        expect(() => parseBoolArray([true], '?', { min_len: 2 }))
            .toThrow('?: length must >= 2');
        
        expect(() => parseBoolArray([true, false, true], '?', { max_len: 2 }))
            .toThrow('?: length must <= 2');
        
        assert.deepEqual(parseBoolArray([true, false, 0], '?'), [true, false, false]);
    });

    it('parseIntArray', () => {
        expect(() => parseIntArray(null as any, '?'))
            .toThrow('?: must be an array');
        
        expect(() => parseIntArray([null] as any, '?'))
            .toThrow('?[0]: must be a int');
        
        expect(() => parseIntArray([null] as any, '?', { allow_bool: true }))
            .toThrow('?[0]: must be a int/boolean');
        
        expect(() => parseIntArray([0], '?', { min: 1 }))
            .toThrow('?[0]: must >= 1');
        
        expect(() => parseIntArray([11], '?', { max: 10 }))
            .toThrow('?[0]: must <= 10');
        
        expect(() => parseIntArray([], '?', { len: 1 }))
            .toThrow('?: length must = 1');
        
        expect(() => parseIntArray([1], '?', { min_len: 2 }))
            .toThrow('?: length must >= 2');
        
        expect(() => parseIntArray([1, 2, 3], '?', { max_len: 2 }))
            .toThrow('?: length must <= 2');
        
        assert.deepEqual(parseIntArray([1, 2, 3], '?'), [1, 2, 3]);
    });

    it('parseIntRange', () => {
        expect(() => parseIntRange(null as any, '?'))
            .toThrow('?: must be an array');
        
        expect(() => parseIntRange([1] as any, '?'))
            .toThrow('?: length must = 2');
        
        expect(() => parseIntRange([null, null] as any, '?'))
            .toThrow('?[0]: must be a int');
        
        expect(() => parseIntRange([9, 1], '?'))
            .toThrow('?: range[0] must < range[1]');
        
        expect(() => parseIntRange([0, 1], '?', { min: 1 }))
            .toThrow('?[0]: must >= 1');
        
        expect(() => parseIntRange([5, 12], '?', { max: 10 }))
            .toThrow('?[1]: must <= 10');
        
        assert.deepEqual(parseIntRange([1, 2], '?'), [1, 2]);        
    });

    it('parseFloatArray', () => {
        expect(() => parseFloatArray(null as any, '?'))
            .toThrow('?: must be an array');
        
        expect(() => parseFloatArray([null] as any, '?'))
            .toThrow('?[0]: must be a float');
        
        expect(() => parseFloatArray([0], '?', { min: 1 }))
            .toThrow('?[0]: must >= 1');
        
        expect(() => parseFloatArray([11], '?', { max: 10 }))
            .toThrow('?[0]: must <= 10');
        
        expect(() => parseFloatArray([], '?', { len: 1 }))
            .toThrow('?: length must = 1');
        
        expect(() => parseFloatArray([1], '?', { min_len: 2 }))
            .toThrow('?: length must >= 2');
        
        expect(() => parseFloatArray([1, 2, 3], '?', { max_len: 2 }))
            .toThrow('?: length must <= 2');
        
        assert.deepEqual(parseFloatArray([-1.0, 2.2, 3.3], '?'), [-1.0, 2.2, 3.3]);
    });

    it('parseFloatRange', () => {
        expect(() => parseFloatRange(null as any, '?'))
            .toThrow('?: must be an array');
        
        expect(() => parseFloatRange([1] as any, '?'))
            .toThrow('?: length must = 2');
        
        expect(() => parseFloatRange([null, null] as any, '?'))
            .toThrow('?[0]: must be a float');
        
        expect(() => parseFloatRange([9, 1], '?'))
            .toThrow('?: range[0] must < range[1]');
        
        expect(() => parseFloatRange([0, 1], '?', { min: 1 }))
            .toThrow('?[0]: must >= 1');
        
        expect(() => parseFloatRange([5, 12], '?', { max: 10 }))
            .toThrow('?[1]: must <= 10');
        
        assert.deepEqual(parseFloatRange([1, 2], '?'), [1, 2]);        
    });

    it('parseString', () => {
        expect(() => parseString(null as any, '?'))
            .toThrow('?: must be a string');
        
        expect(() => parseString('aa', '?', { min_len: 3 }))
            .toThrow('?: length must >= 3');
        
        expect(() => parseString('aaa', '?', { max_len: 2 }))
            .toThrow('?: length must <= 2');
        
        expect(() => parseString('xyz', '?', { regex: /abc/ }))
            .toThrow('?: must match /abc/');
        
        assert.equal(parseString('hello', '?'), 'hello');
    });

    it('parseFile', () => {
        
    });

    it('checkRecord', () => {
        
    });

    it('checkType', () => {
        
    });

    it('parseID', () => {
        expect(() => parseID(null as any, 'Perk', '?'))
            .toThrow('?: must be a ID');
        
        expect(() => parseID('aa', 'Perk', '?'))
            .toThrow('?: must start with "Perk"');
        
        expect(() => parseID('Perk.Aaa.^', 'Perk', '?'))
            .toThrow('?: must match ID pattern');

        assert.equal(parseID('Perk.Aaa', 'Perk', '?'), 'Perk.Aaa');
    });

    it('parseIDArray', () => {
        expect(() => parseIDArray(null as any, 'Zone', '?'))
            .toThrow('?: must be an array');
        
        expect(() => parseIDArray([null] as any, 'Zone', '?'))
            .toThrow('?[0]: must be a ID');
        
        expect(() => parseIDArray([], 'Zone', '?', { len: 1 }))
            .toThrow('?: length must = 1');
        
        expect(() => parseIDArray(['Zone.A'], 'Zone', '?', { min_len: 2 }))
            .toThrow('?: length must >= 2');
        
        expect(() => parseIDArray(['Zone.A', 'Zone.B', 'Zone.C'], 'Zone', '?', { max_len: 2 }))
            .toThrow('?: length must <= 2');
        
        assert.deepEqual(
            parseIDArray(['Zone.A', 'Zone.B'], 'Zone', '?'),
            ['Zone.A', 'Zone.B'],
        );
    });
});
