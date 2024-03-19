function [pos_record, vel_record, pe_record, ke_record] = sim_1d(init_pos, steps)
  if (nargin != 2)
    printf("Excatly 2 arguments needed. Usage: sim_id(init_pos, steps)");
    return;
  elseif (!isrow(init_pos))
    error("Initial position should be row vector");
  elseif (!isscalar(steps))
    error("Steps should be scalar");
  endif

  body_n = length(init_pos);
  pos_record = zeros(steps + 1, body_n);
  pos_record(1, :) = init_pos;
  vel_record = zeros(steps + 1, body_n);
  pe_record = zeros(steps + 1, 1);
  pe_record(1) = sum(abs(init_pos));
  ke_record = zeros(steps + 1, 1);

  for step_ind = 2:(steps + 1)
    cur_delta_v = zeros(1, body_n);
      for body_ind = 1:body_n
        cur_pos_template = repmat(pos_record(step_ind - 1, body_ind), 1, body_n);
        cur_delta_v(body_ind) = sum(sign(pos_record(step_ind - 1, :) - cur_pos_template));
      endfor
      vel_record(step_ind, :) = vel_record(step_ind - 1, :) + cur_delta_v;
      pos_record(step_ind, :) = pos_record(step_ind - 1, :) + vel_record(step_ind, :);
      pe_record(step_ind) = sum(abs(pos_record(step_ind, :)));
      ke_record(step_ind) = sum(abs(vel_record(step_ind, :)));
  endfor
 endfunction
