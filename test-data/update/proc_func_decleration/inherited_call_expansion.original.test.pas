unit TestInheritedExpansion;

interface

type
  TBase = class
  public
    constructor Create(const AName: string; ACount: Integer);
    procedure Update(var AValue: Integer; out AErr: string; A, B: Integer); virtual;
    procedure Reset; virtual;
    procedure Already(const AName: string); virtual;
    procedure AlreadyCall(const AName: string); virtual;
  end;

  TChild = class(TBase)
  public
    constructor Create(const AName: string; ACount: Integer);
    procedure Update(var AValue: Integer; out AErr: string; A, B: Integer); override;
    procedure Reset; override;
    procedure Already(const AName: string); override;
    procedure AlreadyCall(const AName: string); override;
  end;

implementation

constructor TChild.Create(const AName: string; ACount: Integer);
begin
  inherited;
end;

procedure TChild.Update(var AValue: Integer; out AErr: string; A, B: Integer);
begin
  inherited;
end;

procedure TChild.Reset;
begin
  inherited;
end;

procedure TChild.Already(const AName: string);
begin
  inherited Already;
end;

procedure TChild.AlreadyCall(const AName: string);
begin
  inherited AlreadyCall(AName);
end;

end.