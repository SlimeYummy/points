import assert from 'node:assert';
import { describe, expect, it } from '@jest/globals';
import { FPS, parseTime, parseTimeArray, parseTimeRange, TimeFragment } from '../src/common';

// prettier-ignore
describe('time', () => {
    it('parseTime', () => {
        expect(() => parseTime(null as any, '?'))
            .toThrow('?: must be a float/time');
        
        expect(() => parseTime(0, '?', { min: 10 }))
            .toThrow('?: must >= 10');
        
        expect(() => parseTime(11, '?', { max: 9 }))
            .toThrow('?: must <= 9');
        
        expect(() => parseTime('9hh', '?'))
            .toThrow('?: invalid time');
        
        assert.equal(parseTime(5, '?'), 5);
        assert.equal(parseTime('5s', '?'), 5);
        assert.equal(parseTime('5min', '?'), 5 * 60);
        assert.equal(parseTime('5h', '?'), 5 * 3600);
        assert.equal(parseTime('1000ms', '?'), 1);
        assert.equal(parseTime('5F', '?'), 5 / FPS);
    });

    it('parseTimeArray', () => {
        expect(() => parseTimeArray(null as any, '?'))
            .toThrow('?: must be an array');
        
        expect(() => parseTimeArray([null] as any, '?'))
            .toThrow('?[0]: must be a float/time');
        
        expect(() => parseTimeArray([], '?', { len: 1 }))
            .toThrow('?: length must = 1');
        
        expect(() => parseTimeArray([1], '?', { min_len: 2 }))
            .toThrow('?: length must >= 2');
        
        expect(() => parseTimeArray([5, 5, 5], '?', { max_len: 2 }))
            .toThrow('?: length must <= 2');
        
        assert.deepEqual(
            parseTimeArray([5, '5s', '5min', '1h', '1000ms', '7F'], '?'),
            [5, 5, 300, 3600, 1, 7 / FPS]
        );
    });

    it('parseTimeRange', () => {
        expect(() => parseTimeRange(null as any, '?'))
            .toThrow('?: must be an array');
        
        expect(() => parseTimeRange([1] as any, '?'))
            .toThrow('?: length must = 2');
        
        expect(() => parseTimeRange([null, null] as any, '?'))
            .toThrow('?[0]: must be a float/time');
        
        expect(() => parseTimeRange([9, 1], '?'))
            .toThrow('?: range[0] must < range[1]');
        
        expect(() => parseTimeRange([0, 1], '?', { min: 1 }))
            .toThrow('?[0]: must >= 1');
        
        expect(() => parseTimeRange([5, 12], '?', { max: 10 }))
            .toThrow('?[1]: must <= 10');
        
        assert.deepEqual(parseTimeRange(['5s', '1min'], '?'), [5, 60]);
    });

    it('TimeFragment.parseArray() - basic', () => {
        expect(() => TimeFragment.parseArray([], '?', { duration: 15 }))
            .toThrow('?: length must >= 1');

        expect(() => TimeFragment.parseArray([[0, 15]], '?', { duration: 15 }))
            .not.toThrow();

        expect(() => TimeFragment.parseArray([[-1, 15]], '?', { duration: 15 }))
            .toThrow('?[0][0]: must >= 0');

        expect(() => TimeFragment.parseArray([[0, 16]], '?', { duration: 15 }))
            .toThrow('?[0][1]: must <= 15');

        expect(() => TimeFragment.parseArray([[1, 15]], '?', { duration: 15 }))
            .toThrow('?: Invalid time fragment (begin)');

        expect(() => TimeFragment.parseArray([[0, 14]], '?', { duration: 15 }))
            .toThrow('?: Invalid time fragment (end)');

        expect(() => TimeFragment.parseArray([[0, 20]], '?', { duration: 15, over_duration: 20 }))
            .not.toThrow();

        expect(() => TimeFragment.parseArray([[0, 21]], '?', { duration: 15, over_duration: 20 }))
            .toThrow('?[0][1]: must <= 20');

        expect(() => TimeFragment.parseArray([[0, 14]], '?', { duration: 15, over_duration: 20 }))
            .toThrow('?: Invalid time fragment (end)');

    });

    it('TimeFragment.parseArray() - advanced', () => {
        const r1 = TimeFragment.parseArray([[0, 15]], '?', { duration: 15 });
        assert.deepEqual(r1, [new TimeFragment(0, 15, 0)]);

        const r2 = TimeFragment.parseArray([[0, 10], [10, 15]], '?', { duration: 15 });
        assert.deepEqual(r2, [
            new TimeFragment(0, 10, 0),
            new TimeFragment(10, 15, 1),
        ]);
        
        const r3 = TimeFragment.parseArray([[0, '20s'], ['5s', '10s']], '?', { duration: 20 });
        assert.deepEqual(r3, [
            new TimeFragment(0, 5, 0),
            new TimeFragment(5, 10, 1),
            new TimeFragment(10, 20, 0),
        ]);
        
        const r4 = TimeFragment.parseArray([[0, 10], [10, 20], [5, 15]], '?', { duration: 20 });
        assert.deepEqual(r4, [
            new TimeFragment(0, 5, 0),
            new TimeFragment(5, 15, 2),
            new TimeFragment(15, 20, 1),
        ]);
        
        const r5 = TimeFragment.parseArray([[8, 10], [0, 15], [14, 20]], '?', { duration: 20 });
        assert.deepEqual(r5, [
            new TimeFragment(0, 14, 1),
            new TimeFragment(14, 20, 2),
        ]);
    });
});
